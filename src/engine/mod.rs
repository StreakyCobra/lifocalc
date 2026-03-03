mod errors;
mod functions;

pub use errors::EngineError;

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

pub fn evaluate_expression(input: &str, base_stack: &[f64]) -> Result<f64, EngineError> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    let mut stack = base_stack.to_vec();
    for token in tokens {
        if let Ok(number) = token.parse::<f64>() {
            stack.push(number);
            continue;
        }

        functions::execute_function(token, &mut stack)?;
    }

    stack.last().copied().ok_or(EngineError::EmptyInput)
}

pub fn evaluate_expression_in_place(input: &str, stack: &mut Vec<f64>) -> Result<f64, EngineError> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    for token in tokens {
        if let Ok(number) = token.parse::<f64>() {
            stack.push(number);
            continue;
        }

        functions::execute_function(token, stack)?;
    }

    stack.last().copied().ok_or(EngineError::EmptyInput)
}

pub fn has_number_token(input: &str) -> bool {
    tokenize(input)
        .iter()
        .any(|token| token.parse::<f64>().is_ok())
}

pub fn format_number(number: f64) -> String {
    number.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numbers_only_input() {
        let numbers = parse_numbers("1 2 3").expect("expected numbers to parse");
        assert_eq!(numbers, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn evaluates_rpn_expression_from_empty_stack() {
        let result = evaluate_expression("12 12 *", &[]).expect("expected expression to evaluate");
        assert_eq!(result, 144.0);
    }

    #[test]
    fn evaluates_rpn_expression_using_existing_stack_values() {
        let result =
            evaluate_expression("*", &[3.0, 4.0]).expect("expected expression to evaluate");
        assert_eq!(result, 12.0);
    }

    #[test]
    fn evaluates_in_place_and_mutates_stack() {
        let mut stack = vec![3.0, 4.0];
        let result = evaluate_expression_in_place("+", &mut stack)
            .expect("expected expression to evaluate in place");

        assert_eq!(result, 7.0);
        assert_eq!(stack, vec![7.0]);
    }

    #[test]
    fn sum_collapses_stack_to_single_total() {
        let mut stack = vec![3.0, 4.0, 5.0];
        let result = evaluate_expression_in_place("sum", &mut stack)
            .expect("expected sum to evaluate in place");

        assert_eq!(result, 12.0);
        assert_eq!(stack, vec![12.0]);
    }

    #[test]
    fn sum_requires_non_empty_stack() {
        let error = evaluate_expression_in_place("sum", &mut vec![])
            .expect_err("expected sum on empty stack to fail");
        assert_eq!(
            error,
            EngineError::StackUnderflow {
                needed: 1,
                available: 0
            }
        );
    }
}
