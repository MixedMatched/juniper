use egg::{define_language, Id};

use crate::JuniperBigRational;

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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use egg::RecExpr;
    use num::{BigInt, BigRational, FromPrimitive};

    use crate::JuniperBigRational;

    use super::MathExpression;

    #[test]
    fn test_me_big_rational_integers() -> Result<()> {
        let from_string_5: RecExpr<MathExpression> = "5".parse()?;
        let mut manual_5 = RecExpr::default();
        manual_5.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(5.into(), 1.into()),
        )));

        assert_eq!(from_string_5, manual_5);

        let from_string_62: RecExpr<MathExpression> = "62".parse()?;
        let mut manual_62 = RecExpr::default();
        manual_62.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(62.into(), 1.into()),
        )));

        assert_eq!(from_string_62, manual_62);

        let from_string_3259872938572490806830928172794675: RecExpr<MathExpression> =
            "3259872938572490806830928172794675".parse()?;
        let mut manual_3259872938572490806830928172794675 = RecExpr::default();
        manual_3259872938572490806830928172794675.add(MathExpression::Constant(
            JuniperBigRational(BigRational::new(
                (3259872938572490806830928172794675 as u128).into(),
                1.into(),
            )),
        ));

        assert_eq!(
            from_string_3259872938572490806830928172794675,
            manual_3259872938572490806830928172794675
        );

        Ok(())
    }

    #[test]
    fn test_me_big_rational_fractions() -> Result<()> {
        let from_string_1_2: RecExpr<MathExpression> = "1/2".parse()?;
        let mut manual_1_2 = RecExpr::default();
        manual_1_2.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(1.into(), 2.into()),
        )));

        assert_eq!(from_string_1_2, manual_1_2);

        let from_string_45_7: RecExpr<MathExpression> = "45/7".parse()?;
        let mut manual_45_7 = RecExpr::default();
        manual_45_7.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(45.into(), 7.into()),
        )));

        assert_eq!(from_string_45_7, manual_45_7);

        let from_string_249894_92305094: RecExpr<MathExpression> = "249894/92305094".parse()?;
        let mut manual_249894_92305094 = RecExpr::default();
        manual_249894_92305094.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(249894.into(), 92305094.into()),
        )));

        assert_eq!(from_string_249894_92305094, manual_249894_92305094);

        let from_string_38495798267937937298473442343248_239583986943252583928989819845: RecExpr<
            MathExpression,
        > = "38495798267937937298473442343248/239583986943252583928989819845".parse()?;
        let mut manual_38495798267937937298473442343248_239583986943252583928989819845 =
            RecExpr::default();
        manual_38495798267937937298473442343248_239583986943252583928989819845.add(
            MathExpression::Constant(JuniperBigRational(BigRational::new(
                BigInt::from_u128(38495798267937937298473442343248)
                    .expect("couldn't create BigInt"),
                BigInt::from_u128(239583986943252583928989819845).expect("couldn't create BigInt"),
            ))),
        );

        assert_eq!(
            from_string_38495798267937937298473442343248_239583986943252583928989819845,
            manual_38495798267937937298473442343248_239583986943252583928989819845
        );

        Ok(())
    }

    #[test]
    fn test_me_big_rational_exponents() -> Result<()> {
        let from_string_1e0: RecExpr<MathExpression> = "1e0".parse()?;
        let mut manual_1e0 = RecExpr::default();
        manual_1e0.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_integer(1.into()),
        )));

        assert_eq!(from_string_1e0, manual_1e0);

        let from_string_5e5: RecExpr<MathExpression> = "5e5".parse()?;
        let mut manual_5e5 = RecExpr::default();
        manual_5e5.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_integer(500000.into()),
        )));

        assert_eq!(from_string_5e5, manual_5e5);

        let from_string_1e10: RecExpr<MathExpression> = "1e10".parse()?;
        let mut manual_1e10 = RecExpr::default();
        manual_1e10.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_integer((10000000000 as u128).into()),
        )));

        assert_eq!(from_string_1e10, manual_1e10);

        let from_string_10e1: RecExpr<MathExpression> = "10e1".parse()?;
        let mut manual_10e1 = RecExpr::default();
        manual_10e1.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_integer(100.into()),
        )));

        assert_eq!(from_string_10e1, manual_10e1);

        let from_string_10e10: RecExpr<MathExpression> = "10e10".parse()?;
        let mut manual_10e10 = RecExpr::default();
        manual_10e10.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_integer((100000000000 as u128).into()),
        )));

        assert_eq!(from_string_10e10, manual_10e10);

        Ok(())
    }

    #[test]
    fn test_me_big_rational_decimals() -> Result<()> {
        let from_string_0_5: RecExpr<MathExpression> = "0.5".parse()?;
        let mut manual_0_5 = RecExpr::default();
        manual_0_5.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(1.into(), 2.into()),
        )));

        assert_eq!(from_string_0_5, manual_0_5);

        let from_string_1_5: RecExpr<MathExpression> = "1.5".parse()?;
        let mut manual_1_5 = RecExpr::default();
        manual_1_5.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(3.into(), 2.into()),
        )));

        assert_eq!(from_string_1_5, manual_1_5);

        let from_string_5_2495892: RecExpr<MathExpression> = "5.2495892".parse()?;
        let mut manual_5_2495892 = RecExpr::default();
        manual_5_2495892.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(52495892.into(), 10000000.into()),
        )));

        assert_eq!(from_string_5_2495892, manual_5_2495892);

        let from_string_34985982_0: RecExpr<MathExpression> = "34985982.0".parse()?;
        let mut manual_34985982_0 = RecExpr::default();
        manual_34985982_0.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_u128(34985982).expect("couldn't create BigRational"),
        )));

        assert_eq!(from_string_34985982_0, manual_34985982_0);

        let from_string_40328502808232098_4828509809830824: RecExpr<MathExpression> =
            "40328502808232098.4828509809830824".parse()?;
        let mut manual_40328502808232098_4828509809830824 = RecExpr::default();
        manual_40328502808232098_4828509809830824.add(MathExpression::Constant(
            JuniperBigRational(BigRational::new(
                BigInt::from_u128(403285028082320984828509809830824)
                    .expect("couldn't create BigInt"),
                BigInt::from_u128(10000000000000000).expect("couldn't create BigInt"),
            )),
        ));

        assert_eq!(
            from_string_40328502808232098_4828509809830824,
            manual_40328502808232098_4828509809830824
        );

        Ok(())
    }

    #[test]
    fn test_me_big_rational_exponent_decimals() -> Result<()> {
        let from_string_0_5e5: RecExpr<MathExpression> = "0.5e5".parse()?;
        let mut manual_0_5e5 = RecExpr::default();
        manual_0_5e5.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_integer(50000.into()),
        )));

        assert_eq!(from_string_0_5e5, manual_0_5e5);

        let from_string_1_5e0: RecExpr<MathExpression> = "1.5e0".parse()?;
        let mut manual_1_5e0 = RecExpr::default();
        manual_1_5e0.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new(3.into(), 2.into()),
        )));

        assert_eq!(from_string_1_5e0, manual_1_5e0);

        let from_string_0_2348923985e5: RecExpr<MathExpression> = "0.2348923985e5".parse()?;
        let mut manual_0_2348923985e5 = RecExpr::default();
        manual_0_2348923985e5.add(MathExpression::Constant(JuniperBigRational(
            BigRational::new((2348923985 as u32).into(), 100000.into()),
        )));

        assert_eq!(from_string_0_2348923985e5, manual_0_2348923985e5);

        let from_string_34985982_0e4: RecExpr<MathExpression> = "34985982.0e4".parse()?;
        let mut manual_34985982_0e4 = RecExpr::default();
        manual_34985982_0e4.add(MathExpression::Constant(JuniperBigRational(
            BigRational::from_u128(349859820000).expect("couldn't create BigRational"),
        )));

        assert_eq!(from_string_34985982_0e4, manual_34985982_0e4);

        let from_string_40328502808232098_4828509809830824e2: RecExpr<MathExpression> =
            "40328502808232098.4828509809830824e2".parse()?;
        let mut manual_40328502808232098_4828509809830824e2 = RecExpr::default();
        manual_40328502808232098_4828509809830824e2.add(MathExpression::Constant(
            JuniperBigRational(BigRational::new(
                BigInt::from_u128(403285028082320984828509809830824)
                    .expect("couldn't create BigInt"),
                BigInt::from_u128(100000000000000).expect("couldn't create BigInt"),
            )),
        ));

        assert_eq!(
            from_string_40328502808232098_4828509809830824e2,
            manual_40328502808232098_4828509809830824e2
        );

        Ok(())
    }

    #[test]
    fn test_me_pi() -> Result<()> {
        let from_string: RecExpr<MathExpression> = "π".parse()?;
        let mut manual = RecExpr::default();
        manual.add(MathExpression::Pi);

        assert_eq!(from_string, manual);

        Ok(())
    }

    #[test]
    fn test_me_variable() -> Result<()> {
        let from_string_x: RecExpr<MathExpression> = "x".parse()?;
        let mut manual_x = RecExpr::default();
        manual_x.add(MathExpression::Variable('x'));

        assert_eq!(from_string_x, manual_x);

        let from_string_y: RecExpr<MathExpression> = "y".parse()?;
        let mut manual_y = RecExpr::default();
        manual_y.add(MathExpression::Variable('y'));

        assert_eq!(from_string_y, manual_y);

        let from_string_λ: RecExpr<MathExpression> = "λ".parse()?;
        let mut manual_λ = RecExpr::default();
        manual_λ.add(MathExpression::Variable('λ'));

        assert_eq!(from_string_λ, manual_λ);

        let from_string_量: RecExpr<MathExpression> = "量".parse()?;
        let mut manual_量 = RecExpr::default();
        manual_量.add(MathExpression::Variable('量'));

        assert_eq!(from_string_量, manual_量);

        Ok(())
    }
}
