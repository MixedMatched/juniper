use egg::{Id, RecExpr};
use num::{traits::Inv, ToPrimitive};

use crate::{JuniperBigRational, MathExpression};

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
