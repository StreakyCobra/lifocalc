use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use std::collections::BTreeMap;

use super::EngineError;
use super::units::{BaseDimension, UnitExpr};

#[derive(Debug, Clone, PartialEq)]
pub enum Magnitude {
    Exact(BigRational),
    Approx(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    magnitude: Magnitude,
    dims: BTreeMap<BaseDimension, i32>,
    display_unit: Option<UnitExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedNumber {
    pub primary: String,
    pub approximation: Option<String>,
}

impl Number {
    pub fn zero() -> Self {
        Self::from_exact(BigRational::zero())
    }

    pub fn from_exact(value: BigRational) -> Self {
        Self {
            magnitude: Magnitude::Exact(value),
            dims: BTreeMap::new(),
            display_unit: None,
        }
    }

    pub fn from_approx(value: f64) -> Result<Self, EngineError> {
        if value.is_finite() {
            Ok(Self {
                magnitude: Magnitude::Approx(value),
                dims: BTreeMap::new(),
                display_unit: None,
            })
        } else {
            Err(EngineError::InvalidApproximateResult)
        }
    }

    pub fn from_parts(
        magnitude: Magnitude,
        dims: BTreeMap<BaseDimension, i32>,
        display_unit: Option<UnitExpr>,
    ) -> Self {
        Self {
            magnitude,
            dims,
            display_unit,
        }
    }

    pub fn magnitude(&self) -> &Magnitude {
        &self.magnitude
    }

    pub fn dims(&self) -> &BTreeMap<BaseDimension, i32> {
        &self.dims
    }

    pub fn display_unit(&self) -> Option<&UnitExpr> {
        self.display_unit.as_ref()
    }

    pub fn is_unitless(&self) -> bool {
        self.dims.is_empty()
    }

    pub fn is_zero(&self) -> bool {
        match &self.magnitude {
            Magnitude::Exact(value) => value.is_zero(),
            Magnitude::Approx(value) => *value == 0.0,
        }
    }

    pub fn to_f64(&self) -> Result<f64, EngineError> {
        match &self.magnitude {
            Magnitude::Exact(value) => value
                .to_f64()
                .filter(|value| value.is_finite())
                .ok_or(EngineError::ApproximateConversionFailed),
            Magnitude::Approx(value) => Ok(*value),
        }
    }

    pub fn into_exact(self) -> Option<BigRational> {
        match self.magnitude {
            Magnitude::Exact(value) if self.dims.is_empty() => Some(value),
            Magnitude::Exact(_) | Magnitude::Approx(_) => None,
        }
    }

    pub fn add(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        lhs.ensure_matching_dims(&rhs)?;

        let dims = lhs.dims.clone();
        let display_unit = lhs.display_unit.clone().or(rhs.display_unit.clone());
        match (lhs.magnitude, rhs.magnitude) {
            (Magnitude::Exact(lhs), Magnitude::Exact(rhs)) => {
                Ok(Self::from_parts(Magnitude::Exact(lhs + rhs), dims, display_unit))
            }
            (lhs, rhs) => Ok(Self::from_parts(
                Magnitude::Approx(magnitude_to_f64(&lhs)? + magnitude_to_f64(&rhs)?),
                dims,
                display_unit,
            )),
        }
    }

    pub fn subtract(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        lhs.ensure_matching_dims(&rhs)?;

        let dims = lhs.dims.clone();
        let display_unit = lhs.display_unit.clone().or(rhs.display_unit.clone());
        match (lhs.magnitude, rhs.magnitude) {
            (Magnitude::Exact(lhs), Magnitude::Exact(rhs)) => {
                Ok(Self::from_parts(Magnitude::Exact(lhs - rhs), dims, display_unit))
            }
            (lhs, rhs) => Ok(Self::from_parts(
                Magnitude::Approx(magnitude_to_f64(&lhs)? - magnitude_to_f64(&rhs)?),
                dims,
                display_unit,
            )),
        }
    }

    pub fn multiply(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        let dims = combine_dims(&lhs.dims, &rhs.dims, 1);
        match (lhs.magnitude, rhs.magnitude) {
            (Magnitude::Exact(lhs), Magnitude::Exact(rhs)) => {
                Ok(Self::from_parts(Magnitude::Exact(lhs * rhs), dims, None))
            }
            (lhs, rhs) => Ok(Self::from_parts(
                Magnitude::Approx(magnitude_to_f64(&lhs)? * magnitude_to_f64(&rhs)?),
                dims,
                None,
            )),
        }
    }

    pub fn divide(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        if rhs.is_zero() {
            return Err(EngineError::DivisionByZero);
        }

        let dims = combine_dims(&lhs.dims, &rhs.dims, -1);
        match (lhs.magnitude, rhs.magnitude) {
            (Magnitude::Exact(lhs), Magnitude::Exact(rhs)) => {
                Ok(Self::from_parts(Magnitude::Exact(lhs / rhs), dims, None))
            }
            (lhs, rhs) => Ok(Self::from_parts(
                Magnitude::Approx(magnitude_to_f64(&lhs)? / magnitude_to_f64(&rhs)?),
                dims,
                None,
            )),
        }
    }

    pub fn approximate(self) -> Result<Self, EngineError> {
        Ok(Self::from_parts(
            Magnitude::Approx(self.to_f64()?),
            self.dims,
            self.display_unit,
        ))
    }

    pub fn unary_float_op(
        self,
        op_name: &'static str,
        op: impl FnOnce(f64) -> f64,
    ) -> Result<Self, EngineError> {
        let input = self.to_f64()?;
        let result = op(input);
        if result.is_finite() {
            Ok(Self::from_parts(
                Magnitude::Approx(result),
                self.dims,
                self.display_unit,
            ))
        } else {
            Err(EngineError::InvalidApproximateOperation(op_name))
        }
    }

    fn ensure_matching_dims(&self, other: &Self) -> Result<(), EngineError> {
        if self.dims == other.dims {
            Ok(())
        } else {
            Err(EngineError::IncompatibleUnits)
        }
    }
}

pub fn format_number(number: &Number) -> String {
    match &number.magnitude {
        Magnitude::Exact(number) => {
            if number.is_integer() {
                number.to_integer().to_string()
            } else {
                format!("{}/{}", number.numer(), number.denom())
            }
        }
        Magnitude::Approx(number) => format!("{number}f"),
    }
}

pub fn format_number_parts(number: &Number) -> FormattedNumber {
    FormattedNumber {
        primary: format_number(number),
        approximation: match &number.magnitude {
            Magnitude::Exact(_) => number.to_f64().ok().map(format_approximate),
            Magnitude::Approx(_) => None,
        },
    }
}

fn format_approximate(value: f64) -> String {
    format!("{value}f")
}

pub fn pow10(exponent: usize) -> BigInt {
    BigInt::from(10u8).pow(exponent as u32)
}

fn magnitude_to_f64(magnitude: &Magnitude) -> Result<f64, EngineError> {
    match magnitude {
        Magnitude::Exact(value) => value
            .to_f64()
            .filter(|value| value.is_finite())
            .ok_or(EngineError::ApproximateConversionFailed),
        Magnitude::Approx(value) => Ok(*value),
    }
}

fn combine_dims(
    lhs: &BTreeMap<BaseDimension, i32>,
    rhs: &BTreeMap<BaseDimension, i32>,
    rhs_sign: i32,
) -> BTreeMap<BaseDimension, i32> {
    let mut dims = lhs.clone();
    for (dimension, exponent) in rhs {
        let next = dims.get(dimension).copied().unwrap_or_default() + (*exponent * rhs_sign);
        if next == 0 {
            dims.remove(dimension);
        } else {
            dims.insert(*dimension, next);
        }
    }
    dims
}

#[cfg(test)]
mod tests {
    use super::{FormattedNumber, Magnitude, Number, format_number_parts};
    use num_rational::BigRational;
    use std::collections::BTreeMap;

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
        let number = Number::from_parts(Magnitude::Approx(0.5), BTreeMap::new(), None);

        assert_eq!(
            format_number_parts(&number),
            FormattedNumber {
                primary: "0.5f".to_string(),
                approximation: None,
            }
        );
    }

    #[test]
    fn multiplying_unitless_values_keeps_unitless_dimensions() {
        let value = Number::multiply(
            Number::from_exact(BigRational::from_integer(2.into())),
            Number::from_exact(BigRational::from_integer(3.into())),
        )
        .expect("expected multiplication to succeed");

        assert!(value.is_unitless());
        assert_eq!(super::format_number(&value), "6");
    }
}
