use egg::{
    define_language, merge_option, Analysis, DidMerge, EGraph, Id, Language, PatternAst, RecExpr,
};
use num::{
    bigint::{ParseBigIntError, ToBigInt},
    pow::Pow,
    traits::Inv,
    BigInt, BigRational, BigUint, FromPrimitive, ToPrimitive,
};
use std::str::FromStr;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
pub struct JuniperBigRational(BigRational);

#[derive(Debug)]
pub enum ParseBigRationalError {
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
        // exponent/decimal numbers (e.g. 1.68493e15)
        if s.contains("e") && s.contains(".") {
            if let Some((mantissa, exponent)) = s.split_once("e") {
                let exponent_bigint = exponent.parse::<BigUint>()?;
                let pow: BigInt = Pow::pow(Into::<BigInt>::into(10), exponent_bigint);
                if let Some((mantissa, decimal)) = mantissa.split_once(".") {
                    let mantissa_bigint = mantissa.parse::<BigInt>()?;
                    let decimal_bigint = decimal.parse::<BigInt>()?;
                    let decimal_rational = BigRational::new(
                        decimal_bigint,
                        Pow::pow(Into::<BigInt>::into(10), decimal.len()),
                    );
                    Ok(JuniperBigRational(
                        (decimal_rational + mantissa_bigint) * pow.to_bigint().unwrap(),
                    ))
                } else {
                    Err(ParseBigRationalError::Invalid)
                }
            } else {
                Err(ParseBigRationalError::Invalid)
            }
        }
        // decimal numbers (e.g. 2.58486)
        else if let Some((mantissa, decimal)) = s.split_once(".") {
            let mantissa_bigint = mantissa.parse::<BigInt>()?;
            let decimal_bigint = decimal.parse::<BigInt>()?;
            let decimal_rational = BigRational::new(
                decimal_bigint,
                Pow::pow(Into::<BigInt>::into(10), decimal.len()),
            );
            Ok(JuniperBigRational(decimal_rational + mantissa_bigint))
        }
        // exponent numbers (e.g. 5e55)
        else if let Some((mantissa, exponent)) = s.split_once("e") {
            let mantissa_bigint = mantissa.parse::<BigInt>()?;
            let exponent_bigint = exponent.parse::<BigUint>()?;
            let pow: BigInt = Pow::pow(Into::<BigInt>::into(10), exponent_bigint);
            Ok(JuniperBigRational(BigRational::new(
                mantissa_bigint * pow.to_bigint().unwrap(),
                1.into(),
            )))
        }
        // fractional numbers (e.g. 1/2)
        else if let Some((num, denom)) = s.split_once("/") {
            Ok(JuniperBigRational(BigRational::new(
                num.parse::<BigInt>()?,
                denom.parse::<BigInt>()?,
            )))
        }
        // integers (e.g. 2999568)
        else {
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
    pub enum MathExpression {
        Constant(JuniperBigRational),
        "π" = Pi,
        Variable(char),
        ":=" = Assign([Id; 2]),
        "=" = Eq([Id; 2]),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "^" = Pow([Id; 2]),
        "sqrt" = Sqrt(Id),
        "-" = Neg(Id),
        "inv" = Inv(Id),
        "sin" = Sin(Id),
        "cos" = Cos(Id),
        "anti-d" = Antiderivative([Id; 2]),
        "d" = Derivative([Id; 2]),
        "int" = Integral([Id; 4]),
    }
}

#[derive(Default, Clone)]
pub struct ConstantFold;

impl Analysis<MathExpression> for ConstantFold {
    type Data = Option<(JuniperBigRational, PatternAst<MathExpression>)>;

    fn make(
        egraph: &mut EGraph<MathExpression, ConstantFold>,
        enode: &MathExpression,
    ) -> Self::Data {
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
            MathExpression::Inv(a) => (
                JuniperBigRational({
                    let val = x(a)?.0;
                    if *val.numer() == BigInt::ZERO {
                        BigRational::new(0.into(), 1.into())
                    } else {
                        val.inv()
                    }
                }),
                format!("(inv {})", x(a)?).parse().unwrap(),
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

pub fn approximate(re: &RecExpr<MathExpression>, id: &Id) -> Option<f64> {
    match &re[*id] {
        MathExpression::Constant(JuniperBigRational(big_rat)) => big_rat.to_f64(),
        MathExpression::Variable(_) => None,
        MathExpression::Pi => Some(std::f64::consts::PI),
        MathExpression::Assign(_) => None,
        MathExpression::Eq(_) => None, // maybe in the future?
        MathExpression::Add([a, b]) => Some(approximate(re, &a)? + approximate(re, &b)?),
        MathExpression::Sub([a, b]) => Some(approximate(re, &a)? - approximate(re, &b)?),
        MathExpression::Mul([a, b]) => Some(approximate(re, &a)? * approximate(re, &b)?),
        MathExpression::Div([a, b]) => Some(approximate(re, &a)? / approximate(re, &b)?),
        MathExpression::Pow([a, b]) => Some(approximate(re, &a)?.powf(approximate(re, &b)?)),
        MathExpression::Sqrt(n) => Some(approximate(re, &n)?.sqrt()),
        MathExpression::Neg(n) => Some(-approximate(re, &n)?),
        MathExpression::Inv(n) => Some({
            let approximation = approximate(re, &n)?;
            if approximation == 0.0 {
                0.0
            } else {
                approximation.inv()
            }
        }),
        MathExpression::Sin(n) => Some(approximate(re, &n)?.sin()),
        MathExpression::Cos(n) => Some(approximate(re, &n)?.cos()),
        MathExpression::Antiderivative(_) => None,
        MathExpression::Derivative(_) => None,
        MathExpression::Integral(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
