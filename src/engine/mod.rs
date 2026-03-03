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

        apply_function(token, &mut stack)?;
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

        apply_function(token, stack)?;
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

fn apply_function(name: &str, stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let Some(function) = find_function(name) else {
        return Err(EngineError::UnknownToken(name.to_string()));
    };

    validate_arity(function.arity, stack.len())?;
    (function.evaluate)(stack)
}

fn find_function(name: &str) -> Option<&'static FunctionDef> {
    FUNCTIONS.iter().find(|function| function.name == name)
}

fn validate_arity(arity: Arity, available: usize) -> Result<(), EngineError> {
    match arity {
        Arity::Exact(needed) | Arity::AtLeast(needed) if available < needed => {
            Err(EngineError::StackUnderflow { needed, available })
        }
        _ => Ok(()),
    }
}

fn evaluate_add(stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(lhs + rhs);
    Ok(())
}

fn evaluate_subtract(stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(lhs - rhs);
    Ok(())
}

fn evaluate_multiply(stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(lhs * rhs);
    Ok(())
}

fn evaluate_divide(stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    if rhs == 0.0 {
        return Err(EngineError::DivisionByZero);
    }

    stack.push(lhs / rhs);
    Ok(())
}

fn evaluate_sum(stack: &mut Vec<f64>) -> Result<(), EngineError> {
    let value: f64 = stack.iter().sum();
    stack.clear();
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

#[derive(Debug, Clone, Copy)]
enum Arity {
    Exact(usize),
    AtLeast(usize),
}

#[derive(Debug, Clone, Copy)]
struct FunctionDef {
    name: &'static str,
    arity: Arity,
    evaluate: fn(&mut Vec<f64>) -> Result<(), EngineError>,
}

const FUNCTIONS: &[FunctionDef] = &[
    FunctionDef {
        name: "+",
        arity: Arity::Exact(2),
        evaluate: evaluate_add,
    },
    FunctionDef {
        name: "-",
        arity: Arity::Exact(2),
        evaluate: evaluate_subtract,
    },
    FunctionDef {
        name: "*",
        arity: Arity::Exact(2),
        evaluate: evaluate_multiply,
    },
    FunctionDef {
        name: "/",
        arity: Arity::Exact(2),
        evaluate: evaluate_divide,
    },
    FunctionDef {
        name: "sum",
        arity: Arity::AtLeast(1),
        evaluate: evaluate_sum,
    },
];

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
