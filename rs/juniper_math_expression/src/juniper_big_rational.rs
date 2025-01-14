use std::str::FromStr;

use num::{bigint::{ParseBigIntError, ToBigInt}, pow::Pow, BigInt, BigRational, BigUint};


#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
pub struct JuniperBigRational(pub BigRational);

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