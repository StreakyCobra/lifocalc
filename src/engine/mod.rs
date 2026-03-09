mod errors;
mod eval;
mod functions;
mod number;
mod parse;
mod units;

pub use errors::EngineError;
pub use eval::{
    EvalOptions,
    evaluate_expression,
    evaluate_expression_in_place,
    evaluate_expression_in_place_with_options,
    evaluate_expression_stack,
    evaluate_expression_stack_with_options,
    evaluate_expression_with_options,
};
pub use number::{FormattedNumber, Magnitude, Number, format_number, format_number_parts};
pub use parse::{has_number_token, is_numbers_only, parse_number, parse_numbers, parse_unit_spec, tokenize};
pub use units::{BaseDimension, UnitDef, UnitExpr};
