use anyhow::Result;
use egg::{
    define_language, merge_option, rewrite, Analysis, AstSize, DidMerge, EGraph, Extractor, Id,
    Language, PatternAst, RecExpr, Rewrite, Runner, Subst,
};
use num::FromPrimitive;
use num::{bigint::ParseBigIntError, pow::Pow, BigInt, BigRational, ToPrimitive};
use std::io;
use std::str::FromStr;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
struct JuniperBigRational(BigRational);

#[derive(Debug)]
enum ParseBigRationalError {
    Invalid,
}

impl From<ParseBigIntError> for ParseBigRationalError {
    fn from(_: ParseBigIntError) -> Self {
        ParseBigRationalError::Invalid
    }
}

impl FromStr for JuniperBigRational {
    type Err = ParseBigRationalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((num, denom)) = s.split_once("/") {
            Ok(JuniperBigRational(BigRational::new(
                num.parse::<BigInt>()?,
                denom.parse::<BigInt>()?,
            )))
        } else {
            Ok(JuniperBigRational(BigRational::new(
                s.parse::<BigInt>()?,
                1.into(),
            )))
        }
    }
}

impl std::fmt::Display for JuniperBigRational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

define_language! {
    enum MathExpression {
        Constant(JuniperBigRational),
        Variable(char),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "^" = Pow([Id; 2]),
        "root" = Root([Id; 2]),
        "-" = Neg(Id),
        "sin" = Sin(Id),
        "cos" = Cos(Id),
        "anti-d" = Antiderivative([Id; 2]),
        "d" = Derivative([Id; 2]),
        "int" = Integral([Id; 4]),
    }
}

#[derive(Default)]
pub struct ConstantFold;
impl Analysis<MathExpression> for ConstantFold {
    type Data = Option<(JuniperBigRational, PatternAst<MathExpression>)>;

    fn make(egraph: &EGraph<MathExpression, ConstantFold>, enode: &MathExpression) -> Self::Data {
        let x = |i: &Id| egraph[*i].data.as_ref().map(|d| d.0.clone());
        Some(match enode {
            MathExpression::Constant(c) => (c.clone(), format!("{}", c).parse().unwrap()),
            MathExpression::Add([a, b]) => (
                JuniperBigRational(x(a)?.0 + x(b)?.0),
                format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
            ),
            MathExpression::Sub([a, b]) => (
                JuniperBigRational(x(a)?.0 - x(b)?.0),
                format!("(- {} {})", x(a)?, x(b)?).parse().unwrap(),
            ),
            MathExpression::Mul([a, b]) => (
                JuniperBigRational(x(a)?.0 * x(b)?.0),
                format!("(* {} {})", x(a)?, x(b)?).parse().unwrap(),
            ),
            MathExpression::Div([a, b])
                if x(b) != Some(JuniperBigRational(BigRational::new(0.into(), 1.into()))) =>
            {
                (
                    JuniperBigRational(x(a)?.0 / x(b)?.0),
                    format!("(/ {} {})", x(a)?, x(b)?).parse().unwrap(),
                )
            }
            MathExpression::Pow([a, b])
                if x(a) != Some(JuniperBigRational(BigRational::new(0.into(), 1.into()))) =>
            {
                let exponent = x(b)?.0;
                if exponent.denom() == &BigInt::from_i8(1)? {
                    (
                        JuniperBigRational(x(a)?.0.pow(exponent.numer())),
                        format!("(^ {} {})", x(a)?, x(b)?).parse().unwrap(),
                    )
                } else {
                    return None;
                }
            }
            MathExpression::Neg(a) => (
                JuniperBigRational(-x(a)?.0),
                format!("(- {})", x(a)?).parse().unwrap(),
            ),
            _ => return None,
        })
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        merge_option(to, from, |a, b| {
            assert_eq!(a.0, b.0, "Merged non-equal constants");
            DidMerge(false, false)
        })
    }

    fn modify(egraph: &mut EGraph<MathExpression, ConstantFold>, id: Id) {
        let data = egraph[id].data.clone();
        if let Some((c, pat)) = data {
            if egraph.are_explanations_enabled() {
                egraph.union_instantiations(
                    &pat,
                    &format!("{}", c).parse().unwrap(),
                    &Default::default(),
                    "constant_fold".to_string(),
                );
            } else {
                let added = egraph.add(MathExpression::Constant(c));
                egraph.union(id, added);
            }
            // to not prune, comment this out
            egraph[id].nodes.retain(|n| n.is_leaf());

            #[cfg(debug_assertions)]
            egraph[id].assert_unique_leaves();
        }
    }
}

fn approximate(re: &RecExpr<MathExpression>, id: &Id) -> Option<f64> {
    match &re[*id] {
        MathExpression::Constant(JuniperBigRational(big_rat)) => big_rat.to_f64(),
        MathExpression::Variable(_) => None,
        MathExpression::Add([a, b]) => Some(approximate(re, &a)? + approximate(re, &b)?),
        MathExpression::Sub([a, b]) => Some(approximate(re, &a)? - approximate(re, &b)?),
        MathExpression::Mul([a, b]) => Some(approximate(re, &a)? * approximate(re, &b)?),
        MathExpression::Div([a, b]) => Some(approximate(re, &a)? / approximate(re, &b)?),
        MathExpression::Pow([a, b]) => Some(approximate(re, &a)?.powf(approximate(re, &b)?)),
        MathExpression::Root([a, b]) => Some(match &re[*b] {
            MathExpression::Constant(JuniperBigRational(big_rat)) => {
                if big_rat.denom() == &BigInt::from_u8(1)? {
                    if let Some(numer) = big_rat.numer().to_i8() {
                        match numer {
                            1 => approximate(re, &a)?,
                            2 => approximate(re, &a)?.sqrt(),
                            3 => approximate(re, &a)?.cbrt(),
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        }),
        MathExpression::Neg(n) => Some(-approximate(re, &n)?),
        MathExpression::Sin(n) => Some(approximate(re, &n)?.sin()),
        MathExpression::Cos(n) => Some(approximate(re, &n)?.cos()),
        MathExpression::Antiderivative(_) => None,
        MathExpression::Derivative(_) => None,
        MathExpression::Integral(_) => None,
    }
}

fn is_atomic(re: &RecExpr<MathExpression>, id: &Id) -> bool {
    match &re[*id] {
        MathExpression::Constant(_) => true,
        MathExpression::Variable(_) => true,
        _ => false,
    }
}

fn is_const(var: &str) -> impl Fn(&mut EGraph<MathExpression, ConstantFold>, Id, &Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| egraph[subst[var]].data.is_some()
}

fn main() -> Result<()> {
    let rules: &[Rewrite<MathExpression, ConstantFold>] = &[
        rewrite!("comm-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rewrite!("comm-mul"; "(* ?x ?y)" => "(* ?y ?x)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
        rewrite!("add-zero"; "(+ ?x 0)" => "?x"),
        rewrite!("mul-zero"; "(* ?x 0)" => "0"),
        rewrite!("mul-one"; "(* ?x 1)" => "?x"),
        rewrite!("double-negative"; "(- (- ?x))" => "?x"),
        rewrite!("add-negative"; "(+ ?x (- ?y))" => "(- ?x ?y)"),
        rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),
        rewrite!("distribute"; "(* ?a (+ ?b ?c))" => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("factor"; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
        rewrite!("add-mul"; "(+ ?a ?a)" => "(* ?a 2)"),
        rewrite!("mul-pow"; "(* ?a ?a)" => "(^ ?a 2)"),
        rewrite!("pow-mul"; "(* (^ ?a ?b) (^ ?a ?c))" => "(^ ?a (+ ?b ?c))"),
        rewrite!("pow-zero"; "(^ ?a 0)" => "1"),
        rewrite!("root-one"; "(root ?x 1)" => "?x"),
    ];

    loop {
        println!("Enter a (lisp-y) expression: ");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let expr: RecExpr<MathExpression> = input.parse()?;

                let runner: Runner<MathExpression, ConstantFold> =
                    Runner::default().with_expr(&expr).run(rules);
                let extractor = Extractor::new(&runner.egraph, AstSize);

                let (_, best_expr) = extractor.find_best(runner.roots[0]);
                println!("{}", best_expr);

                let last_id = Id::from(best_expr.as_ref().len() - 1);
                if !is_atomic(&best_expr, &last_id) {
                    if let Some(approximation) = approximate(&best_expr, &last_id) {
                        println!("≈ {}", approximation);
                    }
                }
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
