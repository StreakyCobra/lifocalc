use super::{EngineError, Number, functions, parse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalOptions {
    pub implicit_conversion: bool,
}

impl Default for EvalOptions {
    fn default() -> Self {
        Self {
            implicit_conversion: true,
        }
    }
}

pub fn evaluate_expression_stack(
    input: &str,
    base_stack: &[Number],
) -> Result<Vec<Number>, EngineError> {
    evaluate_expression_stack_with_options(input, base_stack, EvalOptions::default())
}

pub fn evaluate_expression_stack_with_options(
    input: &str,
    base_stack: &[Number],
    options: EvalOptions,
) -> Result<Vec<Number>, EngineError> {
    let tokens = parse::tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    let mut stack = base_stack.to_vec();
    evaluate_tokens(tokens, &mut stack, options)?;
    if stack.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    Ok(stack)
}

pub fn evaluate_expression(input: &str, base_stack: &[Number]) -> Result<Number, EngineError> {
    evaluate_expression_with_options(input, base_stack, EvalOptions::default())
}

pub fn evaluate_expression_with_options(
    input: &str,
    base_stack: &[Number],
    options: EvalOptions,
) -> Result<Number, EngineError> {
    evaluate_expression_stack_with_options(input, base_stack, options)?
        .last()
        .cloned()
        .ok_or(EngineError::EmptyInput)
}

pub fn evaluate_expression_in_place(
    input: &str,
    stack: &mut Vec<Number>,
) -> Result<Number, EngineError> {
    evaluate_expression_in_place_with_options(input, stack, EvalOptions::default())
}

pub fn evaluate_expression_in_place_with_options(
    input: &str,
    stack: &mut Vec<Number>,
    options: EvalOptions,
) -> Result<Number, EngineError> {
    let tokens = parse::tokenize(input);
    if tokens.is_empty() {
        return Err(EngineError::EmptyInput);
    }

    evaluate_tokens(tokens, stack, options)?;
    stack.last().cloned().ok_or(EngineError::EmptyInput)
}

fn evaluate_tokens(
    tokens: Vec<&str>,
    stack: &mut Vec<Number>,
    options: EvalOptions,
) -> Result<(), EngineError> {
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index];
        if let Ok(number) = parse::parse_number(token) {
            stack.push(number);
            index += 1;
            continue;
        }

        if let Some(unit) = parse::parse_unit_spec(token)? {
            if !options.implicit_conversion && tokens.get(index + 1) != Some(&"in") {
                return Err(EngineError::MissingConversionTarget);
            }

            let value = stack.pop().ok_or(EngineError::StackUnderflow {
                needed: 1,
                available: 0,
            })?;
            stack.push(value.convert_display_unit(unit)?);
            index += 1;
            if tokens.get(index) == Some(&"in") {
                index += 1;
            }
            continue;
        }

        if token == "in" {
            return Err(EngineError::MissingConversionTarget);
        }

        functions::execute_function(token, stack)?;
        index += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{EngineError, format_number};

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

    #[test]
    fn evaluates_unit_addition() {
        let result = evaluate_expression("1[kB] 2[B] +", &[]).expect("expected unit addition");
        assert_eq!(format_number(&result), "1.002[kB]");
    }

    #[test]
    fn applies_implicit_unit_conversion_before_later_math() {
        let result = evaluate_expression("1[MB/s] [kB/s] 2 *", &[])
            .expect("expected conversion shorthand to work");
        assert_eq!(format_number(&result), "2000[kB/s]");
    }

    #[test]
    fn rejects_unit_conversion_for_unitless_value() {
        let error = evaluate_expression("2 [kB]", &[]).expect_err("expected invalid conversion");
        assert_eq!(error, EngineError::IncompatibleUnits);
    }

    #[test]
    fn explicit_in_works_when_implicit_conversion_is_disabled() {
        let result = evaluate_expression_with_options(
            "1[MB/s] [kB/s] in 2 *",
            &[],
            EvalOptions {
                implicit_conversion: false,
            },
        )
        .expect("expected explicit conversion to work");

        assert_eq!(format_number(&result), "2000[kB/s]");
    }
}
