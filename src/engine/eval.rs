use super::{EngineError, functions, parse};

pub fn evaluate_expression(input: &str, base_stack: &[f64]) -> Result<f64, EngineError> {
    let tokens = parse::tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    let mut stack = base_stack.to_vec();
    evaluate_tokens(tokens, &mut stack)?;
    stack.last().copied().ok_or(EngineError::EmptyInput)
}

pub fn evaluate_expression_in_place(input: &str, stack: &mut Vec<f64>) -> Result<f64, EngineError> {
    let tokens = parse::tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    evaluate_tokens(tokens, stack)?;
    stack.last().copied().ok_or(EngineError::EmptyInput)
}

pub fn format_number(number: f64) -> String {
    number.to_string()
}

fn evaluate_tokens(tokens: Vec<&str>, stack: &mut Vec<f64>) -> Result<(), EngineError> {
    for token in tokens {
        if let Ok(number) = parse::parse_number(token) {
            stack.push(number);
            continue;
        }

        functions::execute_function(token, stack)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
