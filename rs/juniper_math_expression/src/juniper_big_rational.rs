use core::fmt;
use std::{error::Error, str::FromStr};

use num::{
    bigint::{ParseBigIntError, ToBigInt},
    pow::Pow,
    BigInt, BigRational, BigUint,
};

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
pub struct JuniperBigRational(pub BigRational);

#[derive(Debug)]
pub enum ParseBigRationalError {
    Invalid,
}

impl fmt::Display for ParseBigRationalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "big rational parsing failed because of invalidity")
    }
}

impl Error for ParseBigRationalError {}

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

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};
    use num::{BigInt, BigRational, FromPrimitive};

    use super::JuniperBigRational;

    #[test]
    fn test_big_rational_integer() -> Result<()> {
        let from_string_5: JuniperBigRational = "5".parse()?;
        let manual_5 = JuniperBigRational(BigRational::new(5.into(), 1.into()));

        assert_eq!(from_string_5, manual_5);

        let from_string_32: JuniperBigRational = "32".parse()?;
        let manual_32 = JuniperBigRational(BigRational::new(32.into(), 1.into()));

        assert_eq!(from_string_32, manual_32);

        let from_string_25622: JuniperBigRational = "25622".parse()?;
        let manual_25622 = JuniperBigRational(BigRational::new(25622.into(), 1.into()));

        assert_eq!(from_string_25622, manual_25622);

        let from_string_1490591339594333332596335959688: JuniperBigRational =
            "1490591339594333332596335959688".parse()?;
        let manual_1490591339594333332596335959688 = JuniperBigRational(
            BigRational::from_u128(1490591339594333332596335959688)
                .expect("couldn't create BigRational"),
        );

        assert_eq!(
            from_string_1490591339594333332596335959688,
            manual_1490591339594333332596335959688
        );

        Ok(())
    }

    #[test]
    fn test_big_rational_fractions() -> Result<()> {
        let from_string_1_2: JuniperBigRational = "1/2".parse()?;
        let manual_1_2 = JuniperBigRational(BigRational::new(1.into(), 2.into()));

        assert_eq!(from_string_1_2, manual_1_2);

        let from_string_45_7: JuniperBigRational = "45/7".parse()?;
        let manual_45_7 = JuniperBigRational(BigRational::new(45.into(), 7.into()));

        assert_eq!(from_string_45_7, manual_45_7);

        let from_string_249894_92305094: JuniperBigRational = "249894/92305094".parse()?;
        let manual_249894_92305094 =
            JuniperBigRational(BigRational::new(249894.into(), 92305094.into()));

        assert_eq!(from_string_249894_92305094, manual_249894_92305094);

        let from_string_38495798267937937298473442343248_239583986943252583928989819845: JuniperBigRational = "38495798267937937298473442343248/239583986943252583928989819845".parse()?;
        let manual_38495798267937937298473442343248_239583986943252583928989819845 =
            JuniperBigRational(BigRational::new(
                BigInt::from_u128(38495798267937937298473442343248)
                    .expect("couldn't create BigInt"),
                BigInt::from_u128(239583986943252583928989819845).expect("couldn't create BigInt"),
            ));

        assert_eq!(
            from_string_38495798267937937298473442343248_239583986943252583928989819845,
            manual_38495798267937937298473442343248_239583986943252583928989819845
        );

        Ok(())
    }

    #[test]
    fn test_big_rational_exponents() -> Result<()> {
        let from_string_1e0: JuniperBigRational = "1e0".parse()?;
        let manual_1e0 = JuniperBigRational(BigRational::from_integer(1.into()));

        assert_eq!(from_string_1e0, manual_1e0);

        let from_string_5e5: JuniperBigRational = "5e5".parse()?;
        let manual_5e5 = JuniperBigRational(BigRational::from_integer(500000.into()));

        assert_eq!(from_string_5e5, manual_5e5);

        let from_string_1e10: JuniperBigRational = "1e10".parse()?;
        let manual_1e10 =
            JuniperBigRational(BigRational::from_integer((10000000000 as u128).into()));

        assert_eq!(from_string_1e10, manual_1e10);

        let from_string_10e1: JuniperBigRational = "10e1".parse()?;
        let manual_10e1 = JuniperBigRational(BigRational::from_integer(100.into()));

        assert_eq!(from_string_10e1, manual_10e1);

        let from_string_10e10: JuniperBigRational = "10e10".parse()?;
        let manual_10e10 =
            JuniperBigRational(BigRational::from_integer((100000000000 as u128).into()));

        assert_eq!(from_string_10e10, manual_10e10);

        Ok(())
    }

    #[test]
    fn test_big_rational_decimals() -> Result<()> {
        let from_string_0_5: JuniperBigRational = "0.5".parse()?;
        let manual_0_5 = JuniperBigRational(BigRational::new(1.into(), 2.into()));

        assert_eq!(from_string_0_5, manual_0_5);

        let from_string_1_5: JuniperBigRational = "1.5".parse()?;
        let manual_1_5 = JuniperBigRational(BigRational::new(3.into(), 2.into()));

        assert_eq!(from_string_1_5, manual_1_5);

        let from_string_5_2495892: JuniperBigRational = "5.2495892".parse()?;
        let manual_5_2495892 =
            JuniperBigRational(BigRational::new(52495892.into(), 10000000.into()));

        assert_eq!(from_string_5_2495892, manual_5_2495892);

        let from_string_34985982_0: JuniperBigRational = "34985982.0".parse()?;
        let manual_34985982_0 = JuniperBigRational(
            BigRational::from_u128(34985982).expect("couldn't create BigRational"),
        );

        assert_eq!(from_string_34985982_0, manual_34985982_0);

        let from_string_40328502808232098_4828509809830824: JuniperBigRational =
            "40328502808232098.4828509809830824".parse()?;
        let manual_40328502808232098_4828509809830824 = JuniperBigRational(BigRational::new(
            BigInt::from_u128(403285028082320984828509809830824).expect("couldn't create BigInt"),
            BigInt::from_u128(10000000000000000).expect("couldn't create BigInt"),
        ));

        assert_eq!(
            from_string_40328502808232098_4828509809830824,
            manual_40328502808232098_4828509809830824
        );

        Ok(())
    }

    #[test]
    fn test_big_rational_exponent_decimals() -> Result<()> {
        let from_string_0_5e5: JuniperBigRational = "0.5e5".parse()?;
        let manual_0_5e5 = JuniperBigRational(BigRational::from_integer(50000.into()));

        assert_eq!(from_string_0_5e5, manual_0_5e5);

        let from_string_1_5e0: JuniperBigRational = "1.5e0".parse()?;
        let manual_1_5e0 = JuniperBigRational(BigRational::new(3.into(), 2.into()));

        assert_eq!(from_string_1_5e0, manual_1_5e0);

        let from_string_0_2348923985e5: JuniperBigRational = "0.2348923985e5".parse()?;
        let manual_0_2348923985e5 =
            JuniperBigRational(BigRational::new((2348923985 as u32).into(), 100000.into()));

        assert_eq!(from_string_0_2348923985e5, manual_0_2348923985e5);

        let from_string_34985982_0e4: JuniperBigRational = "34985982.0e4".parse()?;
        let manual_34985982_0e4 = JuniperBigRational(
            BigRational::from_u128(349859820000).expect("couldn't create BigRational"),
        );

        assert_eq!(from_string_34985982_0e4, manual_34985982_0e4);

        let from_string_40328502808232098_4828509809830824e2: JuniperBigRational =
            "40328502808232098.4828509809830824e2".parse()?;
        let manual_40328502808232098_4828509809830824e2 = JuniperBigRational(BigRational::new(
            BigInt::from_u128(403285028082320984828509809830824).expect("couldn't create BigInt"),
            BigInt::from_u128(100000000000000).expect("couldn't create BigInt"),
        ));

        assert_eq!(
            from_string_40328502808232098_4828509809830824e2,
            manual_40328502808232098_4828509809830824e2
        );

        Ok(())
    }
}
