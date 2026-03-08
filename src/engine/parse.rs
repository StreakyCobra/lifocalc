use num_bigint::BigInt;
use num_rational::BigRational;

use super::{EngineError, Number, number};

pub fn tokenize(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}

pub fn is_numbers_only(input: &str) -> bool {
    let tokens = tokenize(input);
    !tokens.is_empty() && tokens.iter().all(|token| parse_number(token).is_ok())
}

pub fn parse_number(token: &str) -> Result<Number, EngineError> {
    parse_number_impl(token).ok_or_else(|| EngineError::InvalidNumber(token.to_string()))
}

pub fn parse_numbers(input: &str) -> Result<Vec<Number>, EngineError> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    tokens.into_iter().map(parse_number).collect()
}

pub fn has_number_token(input: &str) -> bool {
    tokenize(input).iter().any(|token| parse_number(token).is_ok())
}

fn parse_number_impl(token: &str) -> Option<Number> {
    if token.is_empty() || is_non_finite(token) {
        return None;
    }

    if let Some(number) = parse_fraction(token) {
        return Some(number);
    }

    parse_decimal_or_scientific(token)
}

fn is_non_finite(token: &str) -> bool {
    matches!(token.to_ascii_lowercase().as_str(), "nan" | "inf" | "+inf" | "-inf" | "infinity" | "+infinity" | "-infinity")
}

fn parse_fraction(token: &str) -> Option<Number> {
    let (numerator, denominator) = token.split_once('/')?;
    if denominator.contains('/') {
        return None;
    }

    let numerator = parse_signed_integer(numerator)?;
    let denominator = parse_signed_integer(denominator)?;
    if denominator == BigInt::from(0u8) {
        return None;
    }

    Some(BigRational::new(numerator, denominator))
}

fn parse_decimal_or_scientific(token: &str) -> Option<Number> {
    let (mantissa, exponent) = match token.find(['e', 'E']) {
        Some(index) => {
            let exponent = parse_exponent(&token[index + 1..])?;
            (&token[..index], exponent)
        }
        None => (token, 0),
    };

    let (negative, unsigned) = parse_sign(mantissa);
    if unsigned.is_empty() {
        return None;
    }

    let (integer_part, fractional_part) = match unsigned.split_once('.') {
        Some((integer_part, fractional_part)) => (integer_part, fractional_part),
        None => (unsigned, ""),
    };

    if integer_part.is_empty() && fractional_part.is_empty() {
        return None;
    }

    if !integer_part.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    if !fractional_part.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    if integer_part.is_empty() && fractional_part.is_empty() {
        return None;
    }
    if integer_part.is_empty() && !mantissa.contains('.') {
        return None;
    }

    let digits = format!("{integer_part}{fractional_part}");
    if digits.is_empty() {
        return None;
    }

    let mut numerator = BigInt::parse_bytes(digits.as_bytes(), 10)?;
    if negative {
        numerator = -numerator;
    }

    let scale = fractional_part.len() as i64 - exponent as i64;
    if scale >= 0 {
        let denominator = number::pow10(scale as usize);
        Some(BigRational::new(numerator, denominator))
    } else {
        numerator *= number::pow10((-scale) as usize);
        Some(BigRational::from_integer(numerator))
    }
}

fn parse_exponent(token: &str) -> Option<i32> {
    let (negative, unsigned) = parse_sign(token);
    if unsigned.is_empty() || !unsigned.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    let exponent = unsigned.parse::<i32>().ok()?;
    Some(if negative { -exponent } else { exponent })
}

fn parse_signed_integer(token: &str) -> Option<BigInt> {
    let (negative, unsigned) = parse_sign(token);
    if unsigned.is_empty() || !unsigned.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    let value = BigInt::parse_bytes(unsigned.as_bytes(), 10)?;
    Some(if negative { -value } else { value })
}

fn parse_sign(token: &str) -> (bool, &str) {
    if let Some(rest) = token.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = token.strip_prefix('+') {
        (false, rest)
    } else {
        (false, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::format_number;

    #[test]
    fn parses_numbers_only_input() {
        let numbers = parse_numbers("1 2 3").expect("expected numbers to parse");
        assert_eq!(
            numbers.into_iter().map(|number| format_number(&number)).collect::<Vec<_>>(),
            vec!["1", "2", "3"]
        );
    }

    #[test]
    fn parses_decimal_as_exact_rational() {
        let number = parse_number("0.125").expect("expected decimal to parse");
        assert_eq!(format_number(&number), "1/8");
    }

    #[test]
    fn parses_fraction_and_reduces() {
        let number = parse_number("10/6").expect("expected fraction to parse");
        assert_eq!(format_number(&number), "5/3");
    }

    #[test]
    fn parses_scientific_notation_exactly() {
        let number = parse_number("1.2e3").expect("expected scientific notation to parse");
        assert_eq!(format_number(&number), "1200");
    }

    #[test]
    fn rejects_non_finite_values() {
        let error = parse_number("NaN").expect_err("expected NaN to fail");
        assert_eq!(error, EngineError::InvalidNumber("NaN".to_string()));
    }
}
