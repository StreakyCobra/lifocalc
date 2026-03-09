use std::collections::BTreeMap;

use num_rational::BigRational;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaseDimension {
    Bit,
    Byte,
    Time,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitExpr {
    pub dims: BTreeMap<BaseDimension, i32>,
}

impl UnitExpr {
    pub fn unitless() -> Self {
        Self {
            dims: BTreeMap::new(),
        }
    }

    pub fn is_unitless(&self) -> bool {
        self.dims.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDef {
    pub symbol: &'static str,
    pub factor: BigRational,
    pub dims: BTreeMap<BaseDimension, i32>,
}
