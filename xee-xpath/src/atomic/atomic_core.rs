use ibig::IBig;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::fmt;
use std::rc::Rc;

use xee_schema_type::Xs;

use crate::atomic;
use crate::error;

use super::arithmetic;
use super::comparison;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerType {
    Integer,
    NonPositiveInteger,
    NegativeInteger,
    NonNegativeInteger,
    PositiveInteger,
    Long,
    Int,
    Short,
    Byte,
    UnsignedLong,
    UnsignedInt,
    UnsignedShort,
    UnsignedByte,
}

impl IntegerType {
    fn schema_type(&self) -> Xs {
        match self {
            IntegerType::Integer => Xs::Integer,
            IntegerType::Long => Xs::Long,
            IntegerType::Int => Xs::Int,
            IntegerType::Short => Xs::Short,
            IntegerType::Byte => Xs::Byte,
            IntegerType::UnsignedLong => Xs::UnsignedLong,
            IntegerType::UnsignedInt => Xs::UnsignedInt,
            IntegerType::UnsignedShort => Xs::UnsignedShort,
            IntegerType::UnsignedByte => Xs::UnsignedByte,
            IntegerType::NonPositiveInteger => Xs::NonPositiveInteger,
            IntegerType::NegativeInteger => Xs::NegativeInteger,
            IntegerType::NonNegativeInteger => Xs::NonNegativeInteger,
            IntegerType::PositiveInteger => Xs::PositiveInteger,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringType {
    String,
    NormalizedString,
    Token,
    Language,
    NMTOKEN,
    Name,
    NCName,
    ID,
    IDREF,
    ENTITY,
}

impl StringType {
    fn schema_type(&self) -> Xs {
        match self {
            StringType::String => Xs::String,
            StringType::NormalizedString => Xs::NormalizedString,
            StringType::Token => Xs::Token,
            StringType::Language => Xs::Language,
            StringType::NMTOKEN => Xs::NMTOKEN,
            StringType::Name => Xs::Name,
            StringType::NCName => Xs::NCName,
            StringType::ID => Xs::ID,
            StringType::IDREF => Xs::IDREF,
            StringType::ENTITY => Xs::ENTITY,
        }
    }
}

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, Eq)]
pub enum Atomic {
    String(StringType, Rc<String>),
    Untyped(Rc<String>),
    AnyURI(Rc<String>),
    Boolean(bool),
    Decimal(Decimal),
    Integer(IntegerType, Rc<IBig>),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),
}

impl Atomic {
    pub(crate) fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Atomic::Boolean(b) => Ok(*b),
            // https://www.w3.org/TR/xpath-31/#id-ebv
            // point 4
            Atomic::String(_, s) => Ok(!s.is_empty()),
            Atomic::Untyped(s) => Ok(!s.is_empty()),
            Atomic::AnyURI(s) => Ok(!s.is_empty()),
            // point 5
            Atomic::Integer(_, i) => Ok(!i.is_zero()),
            Atomic::Decimal(d) => Ok(!d.is_zero()),
            Atomic::Float(f) => Ok(!f.is_zero()),
            Atomic::Double(d) => Ok(!d.is_zero()),
        }
    }

    // XXX is this named right? It's consistent with  to_double, to_bool, etc,
    // but inconsistent with the to_string Rust convention
    pub(crate) fn to_str(&self) -> error::Result<&str> {
        match self {
            Atomic::String(_, s) => Ok(s),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn to_string(&self) -> error::Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub(crate) fn string_value(&self) -> error::Result<String> {
        Ok(self.clone().into_canonical())
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
            Atomic::Integer(_, i) => i.is_zero(),
            _ => false,
        }
    }

    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            Atomic::Float(_) | Atomic::Double(_) | Atomic::Decimal(_) | Atomic::Integer(_, _)
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
            Atomic::String(string_type, _) => string_type.schema_type(),
            Atomic::Untyped(_) => Xs::UntypedAtomic,
            Atomic::AnyURI(_) => Xs::AnyURI,
            Atomic::Boolean(_) => Xs::Boolean,
            Atomic::Decimal(_) => Xs::Decimal,
            Atomic::Integer(integer_type, _) => integer_type.schema_type(),
            Atomic::Float(_) => Xs::Float,
            Atomic::Double(_) => Xs::Double,
        }
    }

    pub(crate) fn ensure_base_schema_type(&self, xs: Xs) -> error::Result<()> {
        if self.schema_type().derives_from(xs) {
            Ok(())
        } else {
            Err(error::Error::Type)
        }
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
        write!(
            f,
            "{:?} {}",
            self.schema_type(),
            self.clone().into_canonical()
        )
    }
}

// strings

impl From<String> for Atomic {
    fn from(s: String) -> Self {
        Atomic::String(StringType::String, Rc::new(s))
    }
}

impl From<&str> for Atomic {
    fn from(s: &str) -> Self {
        Atomic::String(StringType::String, Rc::new(s.to_string()))
    }
}

impl From<&String> for Atomic {
    fn from(s: &String) -> Self {
        Atomic::String(StringType::String, Rc::new(s.clone()))
    }
}

impl TryFrom<Atomic> for String {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::String(_, s) => Ok(s.as_ref().clone()),
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

// decimal

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

// integers

impl From<IBig> for Atomic {
    fn from(i: IBig) -> Self {
        Atomic::Integer(IntegerType::Integer, Rc::new(i))
    }
}

impl From<Rc<IBig>> for Atomic {
    fn from(i: Rc<IBig>) -> Self {
        Atomic::Integer(IntegerType::Integer, i)
    }
}

impl TryFrom<Atomic> for Rc<IBig> {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(_, i) => Ok(i),
            _ => Err(error::Error::Type),
        }
    }
}

impl TryFrom<Atomic> for IBig {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(_, i) => Ok(i.as_ref().clone()),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i64> for Atomic {
    fn from(i: i64) -> Self {
        Atomic::Integer(IntegerType::Long, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for i64 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::Long, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i32> for Atomic {
    fn from(i: i32) -> Self {
        Atomic::Integer(IntegerType::Int, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for i32 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::Int, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i16> for Atomic {
    fn from(i: i16) -> Self {
        Atomic::Integer(IntegerType::Short, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for i16 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::Short, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<i8> for Atomic {
    fn from(i: i8) -> Self {
        Atomic::Integer(IntegerType::Byte, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for i8 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::Byte, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u64> for Atomic {
    fn from(i: u64) -> Self {
        Atomic::Integer(IntegerType::UnsignedLong, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for u64 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::UnsignedLong, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u32> for Atomic {
    fn from(i: u32) -> Self {
        Atomic::Integer(IntegerType::UnsignedInt, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for u32 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::UnsignedInt, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u16> for Atomic {
    fn from(i: u16) -> Self {
        Atomic::Integer(IntegerType::UnsignedShort, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for u16 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::UnsignedShort, i) => Ok(i.as_ref().clone().try_into()?),
            _ => Err(error::Error::Type),
        }
    }
}

impl From<u8> for Atomic {
    fn from(i: u8) -> Self {
        Atomic::Integer(IntegerType::UnsignedByte, Rc::new(i.into()))
    }
}

impl TryFrom<Atomic> for u8 {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Integer(IntegerType::UnsignedByte, i) => Ok(i.as_ref().clone().try_into()?),
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
