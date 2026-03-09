use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_rational::BigRational;

use super::EngineError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaseDimension {
    Bit,
    Byte,
    Time,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitExpr {
    pub factor: BigRational,
    pub dims: BTreeMap<BaseDimension, i32>,
    pub text: String,
}

impl UnitExpr {
    pub fn is_unitless(&self) -> bool {
        self.dims.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDef {
    pub symbol: &'static str,
    pub factor: BigRational,
    pub dims: BTreeMap<BaseDimension, i32>,
    pub prefixable: bool,
}

#[derive(Debug, Clone, Copy)]
struct PrefixDef {
    symbol: &'static str,
    exponent: i32,
}

const PREFIXES: &[PrefixDef] = &[
    PrefixDef {
        symbol: "Q",
        exponent: 30,
    },
    PrefixDef {
        symbol: "R",
        exponent: 27,
    },
    PrefixDef {
        symbol: "Y",
        exponent: 24,
    },
    PrefixDef {
        symbol: "Z",
        exponent: 21,
    },
    PrefixDef {
        symbol: "E",
        exponent: 18,
    },
    PrefixDef {
        symbol: "P",
        exponent: 15,
    },
    PrefixDef {
        symbol: "T",
        exponent: 12,
    },
    PrefixDef {
        symbol: "G",
        exponent: 9,
    },
    PrefixDef {
        symbol: "M",
        exponent: 6,
    },
    PrefixDef {
        symbol: "k",
        exponent: 3,
    },
    PrefixDef {
        symbol: "h",
        exponent: 2,
    },
    PrefixDef {
        symbol: "da",
        exponent: 1,
    },
    PrefixDef {
        symbol: "d",
        exponent: -1,
    },
    PrefixDef {
        symbol: "c",
        exponent: -2,
    },
    PrefixDef {
        symbol: "m",
        exponent: -3,
    },
    PrefixDef {
        symbol: "u",
        exponent: -6,
    },
    PrefixDef {
        symbol: "n",
        exponent: -9,
    },
    PrefixDef {
        symbol: "p",
        exponent: -12,
    },
    PrefixDef {
        symbol: "f",
        exponent: -15,
    },
    PrefixDef {
        symbol: "a",
        exponent: -18,
    },
    PrefixDef {
        symbol: "z",
        exponent: -21,
    },
    PrefixDef {
        symbol: "y",
        exponent: -24,
    },
    PrefixDef {
        symbol: "r",
        exponent: -27,
    },
    PrefixDef {
        symbol: "q",
        exponent: -30,
    },
];

const DISPLAY_PREFIXES: &[PrefixDef] = &[
    PrefixDef {
        symbol: "Q",
        exponent: 30,
    },
    PrefixDef {
        symbol: "R",
        exponent: 27,
    },
    PrefixDef {
        symbol: "Y",
        exponent: 24,
    },
    PrefixDef {
        symbol: "Z",
        exponent: 21,
    },
    PrefixDef {
        symbol: "E",
        exponent: 18,
    },
    PrefixDef {
        symbol: "P",
        exponent: 15,
    },
    PrefixDef {
        symbol: "T",
        exponent: 12,
    },
    PrefixDef {
        symbol: "G",
        exponent: 9,
    },
    PrefixDef {
        symbol: "M",
        exponent: 6,
    },
    PrefixDef {
        symbol: "k",
        exponent: 3,
    },
    PrefixDef {
        symbol: "m",
        exponent: -3,
    },
    PrefixDef {
        symbol: "u",
        exponent: -6,
    },
    PrefixDef {
        symbol: "n",
        exponent: -9,
    },
    PrefixDef {
        symbol: "p",
        exponent: -12,
    },
    PrefixDef {
        symbol: "f",
        exponent: -15,
    },
    PrefixDef {
        symbol: "a",
        exponent: -18,
    },
    PrefixDef {
        symbol: "z",
        exponent: -21,
    },
    PrefixDef {
        symbol: "y",
        exponent: -24,
    },
    PrefixDef {
        symbol: "r",
        exponent: -27,
    },
    PrefixDef {
        symbol: "q",
        exponent: -30,
    },
];

pub fn parse_unit_spec(token: &str) -> Result<Option<UnitExpr>, EngineError> {
    let Some(inner) = token.strip_prefix('[').and_then(|token| token.strip_suffix(']')) else {
        return Ok(None);
    };

    parse_unit_expr(inner).map(Some)
}

pub fn parse_unit_expr(source: &str) -> Result<UnitExpr, EngineError> {
    if source.is_empty() {
        return Err(EngineError::InvalidUnitExpression(source.to_string()));
    }

    let mut factor = BigRational::from_integer(1.into());
    let mut dims = BTreeMap::new();
    let mut operator = 1;
    let mut term_count = 0usize;

    for raw_part in source.split(['*', '/']) {
        if raw_part.is_empty() {
            return Err(EngineError::InvalidUnitExpression(source.to_string()));
        }

        let (symbol, exponent) = split_term_exponent(raw_part, source)?;
        let unit = resolve_unit(symbol)?;
        let signed_exponent = exponent * operator;
        factor *= pow_rational(&unit.factor, signed_exponent);

        for (dimension, unit_exponent) in unit.dims {
            let next = dims.get(&dimension).copied().unwrap_or_default() + (unit_exponent * signed_exponent);
            if next == 0 {
                dims.remove(&dimension);
            } else {
                dims.insert(dimension, next);
            }
        }

        term_count += 1;
        let consumed = raw_part.len();
        if source.len() > consumed + term_count - 1 {
            operator = match source.as_bytes().get(consumed + term_count - 1).copied() {
                Some(b'/') => -1,
                _ => 1,
            };
        }
    }

    Ok(UnitExpr {
        factor,
        dims,
        text: source.to_string(),
    })
}

pub fn preferred_display_unit(
    dims: &BTreeMap<BaseDimension, i32>,
    explicit: Option<&UnitExpr>,
    magnitude: f64,
) -> Option<UnitExpr> {
    if dims.is_empty() {
        return None;
    }

    if let Some(explicit) = explicit {
        if &explicit.dims == dims {
            return Some(explicit.clone());
        }
    }

    auto_display_unit(dims, magnitude)
}

pub fn render_unit_expr(dims: &BTreeMap<BaseDimension, i32>) -> String {
    let mut numerator = Vec::new();
    let mut denominator = Vec::new();

    for (dimension, exponent) in dims {
        if *exponent > 0 {
            numerator.push(render_dimension(*dimension, *exponent));
        } else if *exponent < 0 {
            denominator.push(render_dimension(*dimension, -*exponent));
        }
    }

    if numerator.is_empty() {
        numerator.push("1".to_string());
    }

    if denominator.is_empty() {
        numerator.join("*")
    } else {
        format!("{}/{}", numerator.join("*"), denominator.join("*"))
    }
}

fn split_term_exponent<'a>(term: &'a str, source: &str) -> Result<(&'a str, i32), EngineError> {
    let Some((symbol, exponent)) = term.split_once('^') else {
        return Ok((term, 1));
    };
    let exponent = exponent
        .parse::<i32>()
        .ok()
        .filter(|exponent| *exponent > 0)
        .ok_or_else(|| EngineError::InvalidUnitExpression(source.to_string()))?;
    Ok((symbol, exponent))
}

fn resolve_unit(symbol: &str) -> Result<UnitDef, EngineError> {
    if let Some(unit) = exact_unit(symbol) {
        return Ok(unit);
    }

    for prefix in PREFIXES {
        let Some(rest) = symbol.strip_prefix(prefix.symbol) else {
            continue;
        };
        let Some(base) = exact_unit(rest) else {
            continue;
        };
        if !base.prefixable {
            continue;
        }

        return Ok(UnitDef {
            symbol: Box::leak(symbol.to_string().into_boxed_str()),
            factor: base.factor * pow10(prefix.exponent),
            dims: base.dims,
            prefixable: false,
        });
    }

    Err(EngineError::UnknownUnit(symbol.to_string()))
}

fn exact_unit(symbol: &str) -> Option<UnitDef> {
    let dimension = |dimension| {
        let mut dims = BTreeMap::new();
        dims.insert(dimension, 1);
        dims
    };

    match symbol {
        "b" | "bit" | "bits" => Some(UnitDef {
            symbol: "b",
            factor: BigRational::from_integer(1.into()),
            dims: dimension(BaseDimension::Bit),
            prefixable: true,
        }),
        "B" | "byte" | "bytes" => Some(UnitDef {
            symbol: "B",
            factor: BigRational::from_integer(1.into()),
            dims: dimension(BaseDimension::Byte),
            prefixable: true,
        }),
        "s" | "sec" | "second" | "seconds" => Some(UnitDef {
            symbol: "s",
            factor: BigRational::from_integer(1.into()),
            dims: dimension(BaseDimension::Time),
            prefixable: true,
        }),
        "min" | "minute" | "minutes" => Some(UnitDef {
            symbol: "min",
            factor: BigRational::from_integer(60.into()),
            dims: dimension(BaseDimension::Time),
            prefixable: false,
        }),
        "h" | "hr" | "hour" | "hours" => Some(UnitDef {
            symbol: "h",
            factor: BigRational::from_integer(3600.into()),
            dims: dimension(BaseDimension::Time),
            prefixable: false,
        }),
        "d" | "day" | "days" => Some(UnitDef {
            symbol: "d",
            factor: BigRational::from_integer(86400.into()),
            dims: dimension(BaseDimension::Time),
            prefixable: false,
        }),
        _ => None,
    }
}

fn auto_display_unit(dims: &BTreeMap<BaseDimension, i32>, magnitude: f64) -> Option<UnitExpr> {
    let (&first_dimension, &first_exponent) = dims.iter().find(|(_, exponent)| **exponent > 0)?;
    if first_exponent != 1 {
        return Some(UnitExpr {
            factor: BigRational::from_integer(1.into()),
            dims: dims.clone(),
            text: render_unit_expr(dims),
        });
    }

    let mut best: Option<(f64, UnitExpr)> = None;
    for candidate in auto_candidates_for(first_dimension) {
        let display_magnitude = magnitude / rational_to_f64(&candidate.factor)?;
        let score = display_score(display_magnitude);
        match &best {
            Some((best_score, _)) if *best_score <= score => continue,
            _ => {
                let mut full_dims = dims.clone();
                full_dims.insert(first_dimension, 1);
                best = Some((
                    score,
                    UnitExpr {
                        factor: candidate.factor,
                        dims: dims.clone(),
                        text: render_compound_with_scaled_first(dims, first_dimension, candidate.symbol),
                    },
                ));
            }
        }
    }

    best.map(|(_, unit)| unit)
}

fn auto_candidates_for(dimension: BaseDimension) -> Vec<UnitDef> {
    match dimension {
        BaseDimension::Bit => prefixed_candidates("b", BaseDimension::Bit),
        BaseDimension::Byte => prefixed_candidates("B", BaseDimension::Byte),
        BaseDimension::Time => {
            let mut candidates = prefixed_candidates("s", BaseDimension::Time);
            candidates.extend([
                UnitDef {
                    symbol: "min",
                    factor: BigRational::from_integer(60.into()),
                    dims: single_dimension(BaseDimension::Time),
                    prefixable: false,
                },
                UnitDef {
                    symbol: "h",
                    factor: BigRational::from_integer(3600.into()),
                    dims: single_dimension(BaseDimension::Time),
                    prefixable: false,
                },
                UnitDef {
                    symbol: "d",
                    factor: BigRational::from_integer(86400.into()),
                    dims: single_dimension(BaseDimension::Time),
                    prefixable: false,
                },
            ]);
            candidates
        }
    }
}

fn prefixed_candidates(symbol: &'static str, dimension: BaseDimension) -> Vec<UnitDef> {
    let mut candidates = Vec::with_capacity(DISPLAY_PREFIXES.len() + 1);
    candidates.push(UnitDef {
        symbol,
        factor: BigRational::from_integer(1.into()),
        dims: single_dimension(dimension),
        prefixable: true,
    });
    for prefix in DISPLAY_PREFIXES {
        candidates.push(UnitDef {
            symbol: Box::leak(format!("{}{symbol}", prefix.symbol).into_boxed_str()),
            factor: pow10(prefix.exponent),
            dims: single_dimension(dimension),
            prefixable: false,
        });
    }
    candidates
}

fn render_compound_with_scaled_first(
    dims: &BTreeMap<BaseDimension, i32>,
    scaled_dimension: BaseDimension,
    scaled_symbol: &str,
) -> String {
    let mut numerator = Vec::new();
    let mut denominator = Vec::new();

    for (dimension, exponent) in dims {
        let symbol = if *dimension == scaled_dimension && *exponent > 0 {
            scaled_symbol.to_string()
        } else {
            base_symbol(*dimension).to_string()
        };

        if *exponent > 0 {
            numerator.push(render_symbol(symbol, *exponent));
        } else if *exponent < 0 {
            denominator.push(render_symbol(symbol, -*exponent));
        }
    }

    if numerator.is_empty() {
        numerator.push("1".to_string());
    }

    if denominator.is_empty() {
        numerator.join("*")
    } else {
        format!("{}/{}", numerator.join("*"), denominator.join("*"))
    }
}

fn render_dimension(dimension: BaseDimension, exponent: i32) -> String {
    render_symbol(base_symbol(dimension).to_string(), exponent)
}

fn render_symbol(symbol: String, exponent: i32) -> String {
    if exponent == 1 {
        symbol
    } else {
        format!("{symbol}^{exponent}")
    }
}

fn display_score(value: f64) -> f64 {
    let abs = value.abs();
    if (1.0..1000.0).contains(&abs) {
        0.0 + (1000.0 - abs) / 1000.0
    } else if abs == 0.0 {
        1.0
    } else if abs < 1.0 {
        10.0 + (1.0 - abs)
    } else {
        10.0 + (abs / 1000.0)
    }
}

fn single_dimension(dimension: BaseDimension) -> BTreeMap<BaseDimension, i32> {
    let mut dims = BTreeMap::new();
    dims.insert(dimension, 1);
    dims
}

fn base_symbol(dimension: BaseDimension) -> &'static str {
    match dimension {
        BaseDimension::Bit => "b",
        BaseDimension::Byte => "B",
        BaseDimension::Time => "s",
    }
}

fn pow10(exponent: i32) -> BigRational {
    let value = BigInt::from(10u8).pow(exponent.unsigned_abs());
    if exponent >= 0 {
        BigRational::from_integer(value)
    } else {
        BigRational::new(1.into(), value)
    }
}

fn pow_rational(value: &BigRational, exponent: i32) -> BigRational {
    let mut result = BigRational::from_integer(1.into());
    for _ in 0..exponent.unsigned_abs() {
        result *= value.clone();
    }

    if exponent >= 0 {
        result
    } else {
        BigRational::from_integer(1.into()) / result
    }
}

fn rational_to_f64(value: &BigRational) -> Option<f64> {
    use num_traits::ToPrimitive;

    value.to_f64().filter(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::{BaseDimension, parse_unit_expr, parse_unit_spec, preferred_display_unit, render_unit_expr};
    use std::collections::BTreeMap;

    #[test]
    fn parses_prefixed_byte_per_second_unit() {
        let unit = parse_unit_expr("kB/s").expect("expected unit to parse");

        assert_eq!(unit.text, "kB/s");
        assert_eq!(unit.dims.get(&BaseDimension::Byte), Some(&1));
        assert_eq!(unit.dims.get(&BaseDimension::Time), Some(&-1));
    }

    #[test]
    fn parses_bare_unit_spec_token() {
        let unit = parse_unit_spec("[min]")
            .expect("expected token parse to succeed")
            .expect("expected unit spec");

        assert_eq!(unit.text, "min");
    }

    #[test]
    fn renders_generic_compound_unit() {
        let mut dims = BTreeMap::new();
        dims.insert(BaseDimension::Byte, 1);
        dims.insert(BaseDimension::Time, -2);

        assert_eq!(render_unit_expr(&dims), "B/s^2");
    }

    #[test]
    fn prefers_readable_byte_unit() {
        let mut dims = BTreeMap::new();
        dims.insert(BaseDimension::Byte, 1);

        let unit = preferred_display_unit(&dims, None, 1_500.0).expect("expected display unit");
        assert_eq!(unit.text, "kB");
    }
}
