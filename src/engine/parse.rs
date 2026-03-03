use super::EngineError;

pub fn tokenize(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}

pub fn is_numbers_only(input: &str) -> bool {
    let tokens = tokenize(input);
    !tokens.is_empty() && tokens.iter().all(|token| token.parse::<f64>().is_ok())
}

pub fn parse_number(token: &str) -> Result<f64, EngineError> {
    token
        .parse::<f64>()
        .map_err(|_| EngineError::InvalidNumber(token.to_string()))
}

pub fn parse_numbers(input: &str) -> Result<Vec<f64>, EngineError> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    tokens.into_iter().map(parse_number).collect()
}

pub fn has_number_token(input: &str) -> bool {
    tokenize(input)
        .iter()
        .any(|token| token.parse::<f64>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numbers_only_input() {
        let numbers = parse_numbers("1 2 3").expect("expected numbers to parse");
        assert_eq!(numbers, vec![1.0, 2.0, 3.0]);
    }
}
