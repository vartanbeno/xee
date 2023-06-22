use rust_decimal::Decimal;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Integer(Decimal);

impl Integer {
    pub fn new(d: Decimal) -> Self {
        debug_assert!(d.is_integer());
        Self(d)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn to_decimal(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
