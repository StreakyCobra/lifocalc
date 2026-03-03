use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EngineError {
    #[error("input is empty")]
    EmptyInput,
    #[error("invalid number: {0}")]
    InvalidNumber(String),
    #[error("unknown token: {0}")]
    UnknownToken(String),
    #[error("stack underflow: need {needed}, have {available}")]
    StackUnderflow { needed: usize, available: usize },
    #[error("division by zero")]
    DivisionByZero,
}

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

        apply_operator(token, &mut stack)?;
    }

    stack.last().copied().ok_or(EngineError::EmptyInput)
}

pub fn format_number(number: f64) -> String {
    number.to_string()
}

fn apply_operator(operator: &str, stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;

    let value = match operator {
        "+" => lhs + rhs,
        "-" => lhs - rhs,
        "*" => lhs * rhs,
        "/" => {
            if rhs == 0.0 {
                return Err(EngineError::DivisionByZero);
            }

            lhs / rhs
        }
        _ => return Err(EngineError::UnknownToken(operator.to_string())),
    };

    stack.push(value);
    Ok(())
}

fn pop_two(stack: &mut Vec<f64>) -> Result<(f64, f64), EngineError> {
    if stack.len() < 2 {
        return Err(EngineError::StackUnderflow {
            needed: 2,
            available: stack.len(),
        });
    }

    let rhs = stack
        .pop()
        .expect("stack length checked before pop for rhs");
    let lhs = stack
        .pop()
        .expect("stack length checked before pop for lhs");
    Ok((lhs, rhs))
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
        let result = evaluate_expression("*", &[3.0, 4.0]).expect("expected expression to evaluate");
        assert_eq!(result, 12.0);
    }
}
