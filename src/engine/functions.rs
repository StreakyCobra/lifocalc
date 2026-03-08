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
    FunctionDef {
        name: "~",
        arity: Arity::Exact(1),
        evaluate: evaluate_approximate,
    },
    FunctionDef {
        name: "sqrt",
        arity: Arity::Exact(1),
        evaluate: evaluate_sqrt,
    },
    FunctionDef {
        name: "ln",
        arity: Arity::Exact(1),
        evaluate: evaluate_ln,
    },
    FunctionDef {
        name: "exp",
        arity: Arity::Exact(1),
        evaluate: evaluate_exp,
    },
    FunctionDef {
        name: "sin",
        arity: Arity::Exact(1),
        evaluate: evaluate_sin,
    },
    FunctionDef {
        name: "cos",
        arity: Arity::Exact(1),
        evaluate: evaluate_cos,
    },
    FunctionDef {
        name: "tan",
        arity: Arity::Exact(1),
        evaluate: evaluate_tan,
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
    stack.push(Number::add(lhs, rhs)?);
    Ok(())
}

fn evaluate_subtract(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(Number::subtract(lhs, rhs)?);
    Ok(())
}

fn evaluate_multiply(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(Number::multiply(lhs, rhs)?);
    Ok(())
}

fn evaluate_divide(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let (lhs, rhs) = pop_two(stack)?;
    stack.push(Number::divide(lhs, rhs)?);
    Ok(())
}

fn evaluate_sum(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let mut values = std::mem::take(stack).into_iter();
    let first = values.next().ok_or(EngineError::StackUnderflow {
        needed: 1,
        available: 0,
    })?;
    let value = values.try_fold(first, Number::add)?;
    stack.push(value);
    Ok(())
}

fn evaluate_approximate(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.approximate()?);
    Ok(())
}

fn evaluate_sqrt(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.unary_float_op("sqrt", f64::sqrt)?);
    Ok(())
}

fn evaluate_ln(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.unary_float_op("ln", f64::ln)?);
    Ok(())
}

fn evaluate_exp(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.unary_float_op("exp", f64::exp)?);
    Ok(())
}

fn evaluate_sin(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.unary_float_op("sin", f64::sin)?);
    Ok(())
}

fn evaluate_cos(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.unary_float_op("cos", f64::cos)?);
    Ok(())
}

fn evaluate_tan(stack: &mut Vec<Number>) -> Result<(), EngineError> {
    let value = pop_one(stack)?;
    stack.push(value.unary_float_op("tan", f64::tan)?);
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

fn pop_one(stack: &mut Vec<Number>) -> Result<Number, EngineError> {
    stack.pop().ok_or(EngineError::StackUnderflow {
        needed: 1,
        available: 0,
    })
}
