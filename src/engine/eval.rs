use super::{EngineError, Number, functions, parse};

pub fn evaluate_expression_stack(
    input: &str,
    base_stack: &[Number],
) -> Result<Vec<Number>, EngineError> {
    let tokens = parse::tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    let mut stack = base_stack.to_vec();
    evaluate_tokens(tokens, &mut stack)?;
    if stack.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    Ok(stack)
}

pub fn evaluate_expression(input: &str, base_stack: &[Number]) -> Result<Number, EngineError> {
    evaluate_expression_stack(input, base_stack)?
        .last()
        .cloned()
        .ok_or(EngineError::EmptyInput)
}

pub fn evaluate_expression_in_place(
    input: &str,
    stack: &mut Vec<Number>,
) -> Result<Number, EngineError> {
    let tokens = parse::tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    evaluate_tokens(tokens, stack)?;
    stack.last().cloned().ok_or(EngineError::EmptyInput)
}

fn evaluate_tokens(tokens: Vec<&str>, stack: &mut Vec<Number>) -> Result<(), EngineError> {
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
    use crate::engine::format_number;

    fn number(token: &str) -> Number {
        parse::parse_number(token).expect("expected valid number")
    }

    #[test]
    fn evaluates_rpn_expression_from_empty_stack() {
        let result = evaluate_expression("12 12 *", &[]).expect("expected expression to evaluate");
        assert_eq!(result, number("144"));
    }

    #[test]
    fn returns_full_prompt_local_stack_for_inline_expression() {
        let result =
            evaluate_expression_stack(".1 .2 ~", &[]).expect("expected expression to evaluate");

        assert_eq!(result, vec![number("1/10"), number("0.2f")]);
    }

    #[test]
    fn evaluates_rpn_expression_using_existing_stack_values() {
        let result = evaluate_expression("*", &[number("3"), number("4")])
            .expect("expected expression to evaluate");
        assert_eq!(result, number("12"));
    }

    #[test]
    fn evaluates_in_place_and_mutates_stack() {
        let mut stack = vec![number("3"), number("4")];
        let result = evaluate_expression_in_place("+", &mut stack)
            .expect("expected expression to evaluate in place");

        assert_eq!(result, number("7"));
        assert_eq!(stack, vec![number("7")]);
    }

    #[test]
    fn sum_collapses_stack_to_single_total() {
        let mut stack = vec![number("3"), number("4"), number("5")];
        let result = evaluate_expression_in_place("sum", &mut stack)
            .expect("expected sum to evaluate in place");

        assert_eq!(result, number("12"));
        assert_eq!(stack, vec![number("12")]);
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

    #[test]
    fn evaluates_fraction_result_exactly() {
        let result = evaluate_expression("1 3 /", &[]).expect("expected fraction to evaluate");
        assert_eq!(result, number("1/3"));
    }

    #[test]
    fn degrades_mixed_arithmetic_to_approximate() {
        let result = evaluate_expression("1/2 0.5f +", &[]).expect("expected mixed expression");
        assert_eq!(format_number(&result), "1f");
    }

    #[test]
    fn converts_exact_value_to_approximate() {
        let result = evaluate_expression("1 2 / ~", &[]).expect("expected conversion");
        assert_eq!(format_number(&result), "0.5f");
    }

    #[test]
    fn applies_sqrt_as_approximate_operator() {
        let result = evaluate_expression("2 sqrt", &[]).expect("expected sqrt to evaluate");
        assert_eq!(format_number(&result), "1.4142135623730951f");
    }

    #[test]
    fn rejects_invalid_approximate_operation() {
        let error = evaluate_expression("-1 sqrt", &[]).expect_err("expected sqrt to fail");
        assert_eq!(error, EngineError::InvalidApproximateOperation("sqrt"));
    }
}
