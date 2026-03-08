use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};

use super::EngineError;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Exact(BigRational),
    Approx(f64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedNumber {
    pub primary: String,
    pub approximation: Option<String>,
}

impl Number {
    pub fn zero() -> Self {
        Self::Exact(BigRational::zero())
    }

    pub fn from_exact(value: BigRational) -> Self {
        Self::Exact(value)
    }

    pub fn from_approx(value: f64) -> Result<Self, EngineError> {
        if value.is_finite() {
            Ok(Self::Approx(value))
        } else {
            Err(EngineError::InvalidApproximateResult)
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::Exact(value) => value.is_zero(),
            Self::Approx(value) => *value == 0.0,
        }
    }

    pub fn to_f64(&self) -> Result<f64, EngineError> {
        match self {
            Self::Exact(value) => value
                .to_f64()
                .filter(|value| value.is_finite())
                .ok_or(EngineError::ApproximateConversionFailed),
            Self::Approx(value) => Ok(*value),
        }
    }

    pub fn into_exact(self) -> Option<BigRational> {
        match self {
            Self::Exact(value) => Some(value),
            Self::Approx(_) => None,
        }
    }

    pub fn add(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        match (lhs, rhs) {
            (Self::Exact(lhs), Self::Exact(rhs)) => Ok(Self::Exact(lhs + rhs)),
            (lhs, rhs) => Self::from_approx(lhs.to_f64()? + rhs.to_f64()?),
        }
    }

    pub fn subtract(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        match (lhs, rhs) {
            (Self::Exact(lhs), Self::Exact(rhs)) => Ok(Self::Exact(lhs - rhs)),
            (lhs, rhs) => Self::from_approx(lhs.to_f64()? - rhs.to_f64()?),
        }
    }

    pub fn multiply(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        match (lhs, rhs) {
            (Self::Exact(lhs), Self::Exact(rhs)) => Ok(Self::Exact(lhs * rhs)),
            (lhs, rhs) => Self::from_approx(lhs.to_f64()? * rhs.to_f64()?),
        }
    }

    pub fn divide(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        if rhs.is_zero() {
            return Err(EngineError::DivisionByZero);
        }

        match (lhs, rhs) {
            (Self::Exact(lhs), Self::Exact(rhs)) => Ok(Self::Exact(lhs / rhs)),
            (lhs, rhs) => Self::from_approx(lhs.to_f64()? / rhs.to_f64()?),
        }
    }

    pub fn approximate(self) -> Result<Self, EngineError> {
        Self::from_approx(self.to_f64()?)
    }

    pub fn unary_float_op(
        self,
        op_name: &'static str,
        op: impl FnOnce(f64) -> f64,
    ) -> Result<Self, EngineError> {
        let input = self.to_f64()?;
        let result = op(input);
        if result.is_finite() {
            Ok(Self::Approx(result))
        } else {
            Err(EngineError::InvalidApproximateOperation(op_name))
        }
    }
}

pub fn format_number(number: &Number) -> String {
    match number {
        Number::Exact(number) => {
            if number.is_integer() {
                number.to_integer().to_string()
            } else {
                format!("{}/{}", number.numer(), number.denom())
            }
        }
        Number::Approx(number) => format!("{number}f"),
    }
}

pub fn format_number_parts(number: &Number) -> FormattedNumber {
    FormattedNumber {
        primary: format_number(number),
        approximation: match number {
            Number::Exact(_) => number.to_f64().ok().map(format_approximate),
            Number::Approx(_) => None,
        },
    }
}

fn format_approximate(value: f64) -> String {
    format!("{value}f")
}

pub fn pow10(exponent: usize) -> BigInt {
    BigInt::from(10u8).pow(exponent as u32)
}

#[cfg(test)]
mod tests {
    use super::{FormattedNumber, Number, format_number_parts};
    use num_rational::BigRational;

    #[test]
    fn exact_number_includes_approximation_hint() {
        let number = Number::from_exact(BigRational::new(1.into(), 2.into()));

        assert_eq!(
            format_number_parts(&number),
            FormattedNumber {
                primary: "1/2".to_string(),
                approximation: Some("0.5f".to_string()),
            }
        );
    }

    #[test]
    fn approximate_number_stays_single_part() {
        let number = Number::Approx(0.5);

        assert_eq!(
            format_number_parts(&number),
            FormattedNumber {
                primary: "0.5f".to_string(),
                approximation: None,
            }
        );
    }
}
