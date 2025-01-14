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

#[cfg(test)]
mod tests {
    use std::f64::EPSILON;

    use anyhow::Result;

    use super::approximate;

    #[test]
    fn test_approx_none() -> Result<()> {
        let me_var_1 = "x".parse()?;
        let approximation_var_1 = approximate(&me_var_1, &me_var_1.root());

        assert_eq!(approximation_var_1, None);

        let me_var_2 = "(^ 3 x)".parse()?;
        let approximation_var_2 = approximate(&me_var_2, &me_var_2.root());

        assert_eq!(approximation_var_2, None);

        let me_var_3 = "(cos (sin (- (inv (+ x 4)))))".parse()?;
        let approximation_var_3 = approximate(&me_var_3, &me_var_3.root());

        assert_eq!(approximation_var_3, None);

        Ok(())
    }

    #[test]
    fn test_approx_some() -> Result<()> {
        let me_constant = "5.867".parse()?;
        let approximation_constant = approximate(&me_constant, &me_constant.root());
        let actual_constant = 5.867;

        assert!(
            approximation_constant.expect("approximation_constant failed") - actual_constant
                < EPSILON
        );

        let me_pi = "π".parse()?;
        let approximation_pi = approximate(&me_pi, &me_pi.root());
        let actual_pi = std::f64::consts::PI;

        assert!(approximation_pi.expect("approximation_pi failed") - actual_pi < EPSILON);

        let me_add = "(+ 5 5)".parse()?;
        let approximation_add = approximate(&me_add, &me_add.root());
        let actual_add = 10.0;

        assert!(approximation_add.expect("approximation_add failed") - actual_add < EPSILON);

        let me_sub = "(- 8 2)".parse()?;
        let approximation_sub = approximate(&me_sub, &me_sub.root());
        let actual_sub = 6.0;

        assert!(approximation_sub.expect("approximation_sub failed") - actual_sub < EPSILON);

        let me_mul = "(* 3 2)".parse()?;
        let approximation_mul = approximate(&me_mul, &me_mul.root());
        let actual_mul = 6.0;

        assert!(approximation_mul.expect("approximation_mul failed") - actual_mul < EPSILON);

        let me_div = "(/ 3 2)".parse()?;
        let approximation_div = approximate(&me_div, &me_div.root());
        let actual_div = 1.5;

        assert!(approximation_div.expect("approximation_div failed") - actual_div < EPSILON);

        let me_pow = "(^ 3 2)".parse()?;
        let approximation_pow = approximate(&me_pow, &me_pow.root());
        let actual_pow = 9.0;

        assert!(approximation_pow.expect("approximation_pow failed") - actual_pow < EPSILON);

        let me_sqrt = "(sqrt 17)".parse()?;
        let approximation_sqrt = approximate(&me_sqrt, &me_sqrt.root());
        let actual_sqrt = 4.12310562562;

        assert!(approximation_sqrt.expect("approximation_sqrt failed") - actual_sqrt < EPSILON);

        let me_neg = "(- 16)".parse()?;
        let approximation_neg = approximate(&me_neg, &me_neg.root());
        let actual_neg = -16.0;

        assert!(approximation_neg.expect("approximation_neg failed") - actual_neg < EPSILON);

        let me_inv = "(inv 4)".parse()?;
        let approximation_inv = approximate(&me_inv, &me_inv.root());
        let actual_inv = 0.25;

        assert!(approximation_inv.expect("approximation_inv failed") - actual_inv < EPSILON);

        let me_sin = "(sin 3)".parse()?;
        let approximation_sin = approximate(&me_sin, &me_sin.root());
        let actual_sin = 0.14112000806;

        assert!(approximation_sin.expect("approximation_sin failed") - actual_sin < EPSILON);

        let me_cos = "(cos 3)".parse()?;
        let approximation_cos = approximate(&me_cos, &me_cos.root());
        let actual_cos = -0.9899924966;

        assert!(approximation_cos.expect("approximation_cos failed") - actual_cos < EPSILON);

        Ok(())
    }
}
