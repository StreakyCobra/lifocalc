use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum EngineError {
    #[error("input is empty")]
    EmptyInput,
    #[error("invalid number: {0}")]
    InvalidNumber(String),
    #[error("cannot convert exact value to approximate")]
    ApproximateConversionFailed,
    #[error("invalid approximate result")]
    InvalidApproximateResult,
    #[error("invalid approximate operation: {0}")]
    InvalidApproximateOperation(&'static str),
    #[error("unknown token: {0}")]
    UnknownToken(String),
    #[error("stack underflow: need {needed}, have {available}")]
    StackUnderflow { needed: usize, available: usize },
    #[error("division by zero")]
    DivisionByZero,
    #[error("incompatible units")]
    IncompatibleUnits,
    #[error("unknown unit: {0}")]
    UnknownUnit(String),
    #[error("invalid unit expression: {0}")]
    InvalidUnitExpression(String),
    #[error("invalid conversion target")]
    InvalidConversionTarget,
    #[error("explicit 'in' requires a preceding unit conversion")]
    MissingConversionTarget,
}
