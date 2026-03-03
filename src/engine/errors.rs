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
