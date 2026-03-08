use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};

use super::EngineError;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Exact(BigRational),
    Approx(f64),
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

pub fn pow10(exponent: usize) -> BigInt {
    BigInt::from(10u8).pow(exponent as u32)
}
