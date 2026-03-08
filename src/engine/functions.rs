use num_traits::Zero;

use super::{EngineError, Number};

#[derive(Debug, Clone, Copy)]
enum Arity {
    Exact(usize),
    AtLeast(usize),
}

#[derive(Debug, Clone, Copy)]
struct FunctionDef {
    name: &'static str,
    arity: Arity,
    evaluate: fn(&mut Vec<Number>) -> Result<(), EngineError>,
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

pub(super) fn execute_function(name: &str, stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let Some(function) = FUNCTIONS.iter().find(|function| function.name == name) else {
        return Err(EngineError::UnknownToken(name.to_string()));
    };

    validate_arity(function.arity, stack.len())?;
    (function.evaluate)(stack)
}

fn validate_arity(arity: Arity, available: usize) -> Result<(), EngineError> {
    match arity {
        Arity::Exact(needed) | Arity::AtLeast(needed) if available < needed => {
            Err(EngineError::StackUnderflow { needed, available })
        }
        _ => Ok(()),
    }
}

fn evaluate_add(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(lhs + rhs);
    Ok(())
}

fn evaluate_subtract(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(lhs - rhs);
    Ok(())
}

fn evaluate_multiply(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(lhs * rhs);
    Ok(())
}

fn evaluate_divide(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    if rhs.is_zero() {
        return Err(EngineError::DivisionByZero);
    }

    stack.push(lhs / rhs);
    Ok(())
}

fn evaluate_sum(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = stack.iter().cloned().fold(Number::zero(), |acc, value| acc + value);
    stack.clear();
    stack.push(value);
    Ok(())
}

fn pop_two(stack: &mut Vec<Number>) -> Result<(Number, Number), EngineError> {
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
