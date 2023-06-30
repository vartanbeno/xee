use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::fmt;
use std::rc::Rc;

use xee_schema_type::Xs;

use crate::atomic;
use crate::error;

use super::arithmetic;
use super::comparison;

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, Eq)]
pub enum Atomic {
    // strings
    String(Rc<String>),
    Untyped(Rc<String>),
    // boolean
    Boolean(bool),
    // decimal based
    Decimal(Decimal),
    // We use i64 for xs:integer. According to the XML Schema 1.0 spec,
    // xs:integer is derived from xs:decimal. A conforming specification must
    // support 18 digit decimals. Since i64 can hold 19 digits, we are safe to
    // use it and still be conforming. xs:long is aliased to this.
    // The XML Schema 1.1 approaches this differently, but are still within
    // bounds of these restrictions.
    // That said, not all UnsignedLong fit in a i64, so that may lead to trouble
    Integer(i64),
    // machine integers
    Int(i32),
    Short(i16),
    Byte(i8),
    UnsignedLong(u64),
    UnsignedInt(u32),
    UnsignedShort(u16),
    UnsignedByte(u8),
    // floats
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),
}

impl Atomic {
    pub(crate) fn match_type_name(&self, name: &str) -> bool {
        if name == "anyAtomicType" {
            return true;
        }
        match self {
            Atomic::Boolean(_) => name == "boolean",
            Atomic::Integer(_) => name == "integer",
            Atomic::Float(_) => name == "float",
            Atomic::Double(_) => name == "double",
            Atomic::Decimal(_) => name == "decimal",
            Atomic::String(_) => name == "string",
            // TODO: handle all cases instead of this fallback
            _ => false,
        }
    }

    pub(crate) fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Atomic::Integer(i) => Ok(*i != 0),
            Atomic::Decimal(d) => Ok(!d.is_zero()),
            Atomic::Float(f) => Ok(!f.is_zero()),
            Atomic::Double(d) => Ok(!d.is_zero()),
            Atomic::Boolean(b) => Ok(*b),
            Atomic::Int(i) => Ok(*i != 0),
            Atomic::Short(i) => Ok(*i != 0),
            Atomic::Byte(i) => Ok(*i != 0),
            Atomic::UnsignedLong(i) => Ok(*i != 0),
            Atomic::UnsignedInt(i) => Ok(*i != 0),
            Atomic::UnsignedShort(i) => Ok(*i != 0),
            Atomic::UnsignedByte(i) => Ok(*i != 0),
            Atomic::String(s) => Ok(!s.is_empty()),
            Atomic::Untyped(s) => Ok(!s.is_empty()),
        }
    }

    // XXX is this named right? It's consistent with  to_double, to_bool, etc,
    // but inconsistent with the to_string Rust convention
    pub(crate) fn to_str(&self) -> error::Result<&str> {
        match self {
            Atomic::String(s) => Ok(s),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn to_string(&self) -> error::Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub(crate) fn string_value(&self) -> error::Result<String> {
        Ok(match self {
            Atomic::String(s) => s.to_string(),
            Atomic::Untyped(s) => s.to_string(),
            Atomic::Boolean(b) => b.to_string(),
            Atomic::Integer(i) => i.to_string(),
            Atomic::Float(f) => f.to_string(),
            Atomic::Double(d) => d.to_string(),
            Atomic::Decimal(d) => d.to_string(),
            Atomic::Int(i) => i.to_string(),
            Atomic::Short(i) => i.to_string(),
            Atomic::Byte(i) => i.to_string(),
            Atomic::UnsignedLong(i) => i.to_string(),
            Atomic::UnsignedInt(i) => i.to_string(),
            Atomic::UnsignedShort(i) => i.to_string(),
            Atomic::UnsignedByte(i) => i.to_string(),
        })
    }

    pub(crate) fn is_nan(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_nan(),
            Atomic::Double(d) => d.is_nan(),
            _ => false,
        }
    }

    pub(crate) fn is_infinite(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_infinite(),
            Atomic::Double(d) => d.is_infinite(),
            _ => false,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        match self {
            Atomic::Float(f) => f.is_zero(),
            Atomic::Double(d) => d.is_zero(),
            Atomic::Decimal(d) => d.is_zero(),
            Atomic::Integer(i) => i.is_zero(),
            Atomic::Int(i) => i.is_zero(),
            Atomic::Short(i) => i.is_zero(),
            Atomic::Byte(i) => i.is_zero(),
            Atomic::UnsignedLong(i) => i.is_zero(),
            Atomic::UnsignedInt(i) => i.is_zero(),
            Atomic::UnsignedShort(i) => i.is_zero(),
            Atomic::UnsignedByte(i) => i.is_zero(),
            _ => false,
        }
    }

    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            Atomic::Float(_)
                | Atomic::Double(_)
                | Atomic::Decimal(_)
                | Atomic::Integer(_)
                | Atomic::Int(_)
                | Atomic::Short(_)
                | Atomic::Byte(_)
                | Atomic::UnsignedLong(_)
                | Atomic::UnsignedInt(_)
                | Atomic::UnsignedShort(_)
                | Atomic::UnsignedByte(_)
        )
    }

    pub(crate) fn is_true(&self) -> bool {
        if let Atomic::Boolean(b) = self {
            *b
        } else {
            false
        }
    }

    pub(crate) fn schema_type(&self) -> Xs {
        match self {
            Atomic::String(_) => Xs::String,
            Atomic::Untyped(_) => Xs::UntypedAtomic,
            Atomic::Boolean(_) => Xs::Boolean,
            Atomic::Decimal(_) => Xs::Decimal,
            Atomic::Integer(_) => Xs::Integer,
            Atomic::Int(_) => Xs::Int,
            Atomic::Short(_) => Xs::Short,
            Atomic::Byte(_) => Xs::Byte,
            Atomic::UnsignedLong(_) => Xs::UnsignedLong,
            Atomic::UnsignedInt(_) => Xs::UnsignedInt,
            Atomic::UnsignedShort(_) => Xs::UnsignedShort,
            Atomic::UnsignedByte(_) => Xs::UnsignedByte,
            Atomic::Float(_) => Xs::Float,
            Atomic::Double(_) => Xs::Double,
        }
    }

    pub(crate) fn ensure_base_schema_type(&self, xs: Xs) -> error::Result<()> {
        if self.has_base_schema_type(xs) {
            Ok(())
        } else {
            Err(error::Error::Type)
        }
    }

    pub(crate) fn has_base_schema_type(&self, xs: Xs) -> bool {
        self.schema_type().derives_from(xs)
    }

    pub(crate) fn derives_from(&self, other: &Atomic) -> bool {
        self.schema_type().derives_from(other.schema_type())
    }

    pub(crate) fn has_same_schema_type(&self, other: &Atomic) -> bool {
        self.schema_type() == other.schema_type()
    }

    // value comparison as per XPath rules
    pub(crate) fn value_comparison<O>(self, other: Atomic) -> error::Result<bool>
    where
        O: comparison::ComparisonOp,
    {
        comparison::value_comparison_op::<O>(self, other)
    }

    pub(crate) fn arithmetic<O>(self, other: Atomic) -> error::Result<Atomic>
    where
        O: arithmetic::ArithmeticOp,
    {
        arithmetic::arithmetic_op::<O>(self, other)
    }

    pub(crate) fn plus(self) -> error::Result<Atomic> {
        arithmetic::unary_plus(self)
    }

    pub(crate) fn minus(self) -> error::Result<Atomic> {
        arithmetic::unary_minus(self)
    }
}

impl PartialEq for Atomic {
    fn eq(&self, other: &Self) -> bool {
        self.clone()
            .value_comparison::<atomic::EqualOp>(other.clone())
            .unwrap_or(false)
    }
}

impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Atomic::Boolean(b) => write!(f, "{}", b),
            Atomic::Integer(i) => write!(f, "{}", i),
            Atomic::Float(n) => write!(f, "{}", n),
            Atomic::Double(d) => write!(f, "{}", d),
            Atomic::Decimal(d) => write!(f, "{}", d),
            Atomic::String(s) => write!(f, "{}", s),
            Atomic::Untyped(s) => write!(f, "{}", s),
            _ => unreachable!("Cannot exist in output space"),
        }
    }
}

// strings

impl From<String> for Atomic {
    fn from(s: String) -> Self {
        Atomic::String(Rc::new(s))
    }
}

impl From<&str> for Atomic {
    fn from(s: &str) -> Self {
        Atomic::String(Rc::new(s.to_string()))
    }
}

impl From<&String> for Atomic {
    fn from(s: &String) -> Self {
        Atomic::String(Rc::new(s.clone()))
    }
}

impl TryFrom<Atomic> for String {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::String(s) => Ok(s.as_ref().clone()),
            _ => Err(error::Error::Type),
        }
    }
}

// bool

impl From<bool> for Atomic {
    fn from(b: bool) -> Self {
        Atomic::Boolean(b)
    }
}

impl TryFrom<Atomic> for bool {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Boolean(b) => Ok(b),
            _ => Err(error::Error::Type),
        }
    }
}

// decimal based

impl From<Decimal> for Atomic {
    fn from(d: Decimal) -> Self {
        Atomic::Decimal(d)
    }
}

impl TryFrom<Atomic> for Decimal {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Decimal(d) => Ok(d),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i64> for Atomic {
    fn from(i: i64) -> Self {
        Atomic::Integer(i)
    }
}

impl TryFrom<Atomic> for i64 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

// machine integers

// xs:long is xs:integer

impl From<i32> for Atomic {
    fn from(i: i32) -> Self {
        Atomic::Int(i)
    }
}

impl TryFrom<Atomic> for i32 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Int(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i16> for Atomic {
    fn from(i: i16) -> Self {
        Atomic::Short(i)
    }
}

impl TryFrom<Atomic> for i16 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Short(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i8> for Atomic {
    fn from(i: i8) -> Self {
        Atomic::Byte(i)
    }
}

impl TryFrom<Atomic> for i8 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Byte(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u64> for Atomic {
    fn from(i: u64) -> Self {
        Atomic::UnsignedLong(i)
    }
}

impl TryFrom<Atomic> for u64 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::UnsignedLong(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u32> for Atomic {
    fn from(i: u32) -> Self {
        Atomic::UnsignedInt(i)
    }
}

impl TryFrom<Atomic> for u32 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::UnsignedInt(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u16> for Atomic {
    fn from(i: u16) -> Self {
        Atomic::UnsignedShort(i)
    }
}

impl TryFrom<Atomic> for u16 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::UnsignedShort(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u8> for Atomic {
    fn from(i: u8) -> Self {
        Atomic::UnsignedByte(i)
    }
}

impl TryFrom<Atomic> for u8 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::UnsignedByte(i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

// floats

impl From<f32> for Atomic {
    fn from(f: f32) -> Self {
        Atomic::Float(OrderedFloat(f))
    }
}

impl From<OrderedFloat<f32>> for Atomic {
    fn from(f: OrderedFloat<f32>) -> Self {
        Atomic::Float(f)
    }
}

impl TryFrom<Atomic> for f32 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Float(f) => Ok(f.into_inner()),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<f64> for Atomic {
    fn from(f: f64) -> Self {
        Atomic::Double(OrderedFloat(f))
    }
}

impl From<OrderedFloat<f64>> for Atomic {
    fn from(f: OrderedFloat<f64>) -> Self {
        Atomic::Double(f)
    }
}

impl TryFrom<Atomic> for f64 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Double(f) => Ok(f.into_inner()),
            _ => Err(error::Error::Type),
        }
    }
}
