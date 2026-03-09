use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use std::collections::BTreeMap;

use super::EngineError;
use super::units::{BaseDimension, UnitExpr, preferred_display_unit, render_unit_expr};

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

    pub fn scaled(mut self, factor: &BigRational) -> Result<Self, EngineError> {
        self.magnitude = scale_magnitude(self.magnitude, factor)?;
        Ok(self)
    }

    pub fn with_dimensions(mut self, dims: BTreeMap<BaseDimension, i32>) -> Self {
        self.dims = dims;
        self
    }

    pub fn convert_display_unit(mut self, display_unit: UnitExpr) -> Result<Self, EngineError> {
        if self.dims != display_unit.dims {
            return Err(EngineError::IncompatibleUnits);
        }

        self.display_unit = Some(display_unit);
        Ok(self)
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
        let display_unit = if lhs.is_unitless() && !rhs.is_unitless() {
            rhs.display_unit.clone()
        } else if rhs.is_unitless() {
            lhs.display_unit.clone()
        } else {
            None
        };
        match (lhs.magnitude, rhs.magnitude) {
            (Magnitude::Exact(lhs), Magnitude::Exact(rhs)) => {
                Ok(Self::from_parts(Magnitude::Exact(lhs * rhs), dims, display_unit))
            }
            (lhs, rhs) => Ok(Self::from_parts(
                Magnitude::Approx(magnitude_to_f64(&lhs)? * magnitude_to_f64(&rhs)?),
                dims,
                display_unit,
            )),
        }
    }

    pub fn divide(lhs: Self, rhs: Self) -> Result<Self, EngineError> {
        if rhs.is_zero() {
            return Err(EngineError::DivisionByZero);
        }

        let dims = combine_dims(&lhs.dims, &rhs.dims, -1);
        let display_unit = if rhs.is_unitless() {
            lhs.display_unit.clone()
        } else {
            None
        };
        match (lhs.magnitude, rhs.magnitude) {
            (Magnitude::Exact(lhs), Magnitude::Exact(rhs)) => {
                Ok(Self::from_parts(Magnitude::Exact(lhs / rhs), dims, display_unit))
            }
            (lhs, rhs) => Ok(Self::from_parts(
                Magnitude::Approx(magnitude_to_f64(&lhs)? / magnitude_to_f64(&rhs)?),
                dims,
                display_unit,
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
    let chosen_unit = preferred_display_unit(
        &number.dims,
        number.display_unit.as_ref(),
        number.to_f64().unwrap_or_default(),
    );
    let (magnitude, unit_text) = if let Some(unit) = chosen_unit {
        let inverse_factor = BigRational::from_integer(1.into()) / unit.factor.clone();
        (
            scale_magnitude(number.magnitude.clone(), &inverse_factor)
                .unwrap_or_else(|_| number.magnitude.clone()),
            Some(unit.text),
        )
    } else {
        (number.magnitude.clone(), None)
    };

    let mut formatted = format_magnitude(&magnitude, unit_text.is_some());
    if let Some(unit_text) = unit_text {
        formatted.push('[');
        formatted.push_str(&unit_text);
        formatted.push(']');
    } else if !number.dims.is_empty() {
        formatted.push('[');
        formatted.push_str(&render_unit_expr(&number.dims));
        formatted.push(']');
    }
    formatted
}

pub fn format_number_parts(number: &Number) -> FormattedNumber {
    let chosen_unit = preferred_display_unit(
        &number.dims,
        number.display_unit.as_ref(),
        number.to_f64().unwrap_or_default(),
    );
    let approximation = match &number.magnitude {
        Magnitude::Exact(_) => {
            let unit_factor = chosen_unit
                .as_ref()
                .map(|unit| unit.factor.clone())
                .unwrap_or_else(|| BigRational::from_integer(1.into()));
            let scaled = number.to_f64().ok().map(|value| {
                let factor = unit_factor.to_f64().unwrap_or(1.0);
                value / factor
            });
            scaled.map(|value| format_approximate(value, chosen_unit.as_ref().map(|unit| unit.text.as_str())))
        }
        Magnitude::Approx(_) => None,
    };

    FormattedNumber {
        primary: format_number(number),
        approximation,
    }
}

fn format_approximate(value: f64, unit_text: Option<&str>) -> String {
    let mut formatted = format_decimal_with_grouping(&format!("{value}f"));
    if let Some(unit_text) = unit_text {
        formatted.push('[');
        formatted.push_str(unit_text);
        formatted.push(']');
    }
    formatted
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

fn scale_magnitude(magnitude: Magnitude, factor: &BigRational) -> Result<Magnitude, EngineError> {
    match magnitude {
        Magnitude::Exact(value) => Ok(Magnitude::Exact(value * factor.clone())),
        Magnitude::Approx(value) => Ok(Magnitude::Approx(
            value
                * factor
                    .to_f64()
                    .filter(|factor| factor.is_finite())
                    .ok_or(EngineError::ApproximateConversionFailed)?,
        )),
    }
}

fn format_magnitude(magnitude: &Magnitude, prefer_decimal: bool) -> String {
    match magnitude {
        Magnitude::Exact(number) => format_exact_rational(number, prefer_decimal),
        Magnitude::Approx(number) => format_decimal_with_grouping(&format!("{number}f")),
    }
}

fn format_exact_rational(number: &BigRational, prefer_decimal: bool) -> String {
    if number.is_integer() {
        return format_integer_with_grouping(&number.to_integer().to_string());
    }

    if prefer_decimal {
        if let Some(decimal) = format_terminating_decimal(number) {
            return decimal;
        }
    }

    format!(
        "{}/{}",
        format_integer_with_grouping(&number.numer().to_string()),
        format_integer_with_grouping(&number.denom().to_string())
    )
}

fn format_terminating_decimal(number: &BigRational) -> Option<String> {
    let mut denominator = number.denom().clone();
    let two = BigInt::from(2u8);
    let five = BigInt::from(5u8);
    let zero = BigInt::from(0u8);
    let mut twos = 0usize;
    let mut fives = 0usize;

    while (&denominator % &two) == zero {
        denominator /= &two;
        twos += 1;
    }
    while (&denominator % &five) == zero {
        denominator /= &five;
        fives += 1;
    }

    if denominator != BigInt::from(1u8) {
        return None;
    }

    let scale = twos.max(fives);
    let mut numerator = number.numer().clone();
    if twos < scale {
        numerator *= BigInt::from(2u8).pow((scale - twos) as u32);
    }
    if fives < scale {
        numerator *= BigInt::from(5u8).pow((scale - fives) as u32);
    }

    let negative = numerator < BigInt::from(0u8);
    let digits = if negative { (-numerator).to_string() } else { numerator.to_string() };
    if scale == 0 {
        return Some(if negative { format!("-{digits}") } else { digits });
    }

    let padded = if digits.len() <= scale {
        format!("{}{}", "0".repeat(scale + 1 - digits.len()), digits)
    } else {
        digits
    };
    let split_at = padded.len() - scale;
    let result = format!(
        "{}.{}",
        format_integer_with_grouping(&padded[..split_at]),
        &padded[split_at..]
    );
    Some(if negative { format!("-{result}") } else { result })
}

fn format_decimal_with_grouping(value: &str) -> String {
    let Some(number_end) = value.find(|ch: char| !(ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E')))
    else {
        return format_number_text_with_grouping(value);
    };

    let (number_part, suffix) = value.split_at(number_end);
    format!("{}{}", format_number_text_with_grouping(number_part), suffix)
}

fn format_number_text_with_grouping(value: &str) -> String {
    let Some(exponent_index) = value.find(['e', 'E']) else {
        return format_non_exponent_decimal_with_grouping(value);
    };

    let (mantissa, exponent) = value.split_at(exponent_index);
    format!(
        "{}{}",
        format_non_exponent_decimal_with_grouping(mantissa),
        exponent
    )
}

fn format_non_exponent_decimal_with_grouping(value: &str) -> String {
    let sign_len = usize::from(value.starts_with('-') || value.starts_with('+'));
    let (sign, unsigned) = value.split_at(sign_len);

    let Some(decimal_index) = unsigned.find('.') else {
        return format!("{}{}", sign, format_integer_with_grouping(unsigned));
    };

    let (integer, fraction) = unsigned.split_at(decimal_index);
    format!("{}{integer_grouped}{fraction}", sign, integer_grouped = format_integer_with_grouping(integer))
}

fn format_integer_with_grouping(value: &str) -> String {
    let (sign, digits) = if let Some(stripped) = value.strip_prefix('-') {
        ("-", stripped)
    } else if let Some(stripped) = value.strip_prefix('+') {
        ("+", stripped)
    } else {
        ("", value)
    };

    if digits.len() <= 3 {
        return value.to_string();
    }

    let mut formatted = String::with_capacity(value.len() + (digits.len() - 1) / 3);
    formatted.push_str(sign);

    let first_group_len = match digits.len() % 3 {
        0 => 3,
        len => len,
    };
    formatted.push_str(&digits[..first_group_len]);

    let mut index = first_group_len;
    while index < digits.len() {
        formatted.push('\'');
        formatted.push_str(&digits[index..index + 3]);
        index += 3;
    }

    formatted
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

    #[test]
    fn formats_large_integer_with_grouping() {
        let number = Number::from_exact(BigRational::from_integer(43200.into()));

        assert_eq!(super::format_number(&number), "43'200");
    }

    #[test]
    fn formats_large_approximate_with_grouping() {
        let number = Number::from_parts(Magnitude::Approx(43200.5), BTreeMap::new(), None);

        assert_eq!(super::format_number(&number), "43'200.5f");
    }

    #[test]
    fn formats_fraction_parts_with_grouping() {
        let number = Number::from_exact(BigRational::new(43200.into(), 1001.into()));

        assert_eq!(super::format_number(&number), "43'200/1'001");
    }
}
