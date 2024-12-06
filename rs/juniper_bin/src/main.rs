use anyhow::Result;
use egg::{rewrite, AstSize, Extractor, Id, RecExpr, Rewrite, Runner};
use juniper_math_expression::{approximate, ConstantFold, MathExpression};
use std::io;

fn is_atomic(re: &RecExpr<MathExpression>, id: &Id) -> bool {
    match &re[*id] {
        MathExpression::Constant(_) => true,
        MathExpression::Variable(_) => true,
        _ => false,
    }
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
