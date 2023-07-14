use ibig::{ibig, IBig};
use num_traits::Float;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::rc::Rc;

use xee_schema_type::{BaseNumericType, Xs};

use crate::error;

use super::atomic_core as atomic;

impl atomic::Atomic {
    pub(crate) fn parse_atomic<V>(s: &str) -> error::Result<atomic::Atomic>
    where
        Parsed<V>: FromStr,
        V: Into<atomic::Atomic>,
    {
        // TODO: re-establish error. I am not sure what error is returned
        // from the FromStr trait; it should be error::Error but evidently it's
        // not
        s.parse::<Parsed<V>>()
            .map(|p| p.into_inner().into())
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_decimal(s: &str) -> error::Result<Decimal> {
        if s.contains('_') {
            return Err(error::Error::FORG0001);
        }
        s.parse::<Decimal>().map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_integer_number<V>(s: &str) -> error::Result<V>
    where
        V: lexical::FromLexical,
    {
        lexical::parse::<V, _>(s).map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_boolean(s: &str) -> error::Result<bool> {
        match s {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(error::Error::FORG0001),
        }
    }

    // we can't use lexical::parse_float_options::XML as it doesn't allow INF
    // which is allowed by the XML Schema spec

    pub(crate) fn parse_float(s: &str) -> error::Result<f32> {
        let options = lexical::ParseFloatOptionsBuilder::new()
            .inf_string(Some(b"INF"))
            .build()
            .unwrap();
        lexical::parse_with_options::<f32, _, { lexical::format::XML }>(s, &options)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_double(s: &str) -> error::Result<f64> {
        let options = lexical::ParseFloatOptionsBuilder::new()
            .inf_string(Some(b"INF"))
            .build()
            .unwrap();
        lexical::parse_with_options::<f64, _, { lexical::format::XML }>(s, &options)
            .map_err(|_| error::Error::FORG0001)
    }

    // from an atomic type to a canonical representation as a string
    pub(crate) fn to_canonical(&self) -> String {
        match self {
            atomic::Atomic::String(s) => s.as_ref().clone(),
            atomic::Atomic::Untyped(s) => s.as_ref().clone(),
            atomic::Atomic::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            atomic::Atomic::Decimal(d) => {
                if d.is_integer() {
                    // TODO: this could fail if the decimal is too big. Instead
                    // we should relent and use bigint for xs:integer
                    let i: i64 = (*d).try_into().unwrap();
                    i.to_string()
                } else {
                    d.normalize().to_string()
                }
            }
            atomic::Atomic::Integer(i) => i.to_string(),
            atomic::Atomic::Long(i) => i.to_string(),
            atomic::Atomic::Int(i) => i.to_string(),
            atomic::Atomic::Short(i) => i.to_string(),
            atomic::Atomic::Byte(i) => i.to_string(),
            atomic::Atomic::UnsignedLong(i) => i.to_string(),
            atomic::Atomic::UnsignedInt(i) => i.to_string(),
            atomic::Atomic::UnsignedShort(i) => i.to_string(),
            atomic::Atomic::UnsignedByte(i) => i.to_string(),
            atomic::Atomic::NonPositiveInteger(i) => i.to_string(),
            atomic::Atomic::NegativeInteger(i) => i.to_string(),
            atomic::Atomic::NonNegativeInteger(i) => i.to_string(),
            atomic::Atomic::PositiveInteger(i) => i.to_string(),
            atomic::Atomic::Float(OrderedFloat(f)) => canonical_float(*f),
            atomic::Atomic::Double(OrderedFloat(f)) => canonical_float(*f),
        }
    }

    pub(crate) fn cast_to_schema_type(&self, xs: Xs) -> error::Result<atomic::Atomic> {
        // if we try to cast to any atomic type, we're already the correct type
        if xs == Xs::AnyAtomicType {
            // TODO: if we made the cast functions take self, not &self, we
            // could make this cheaper
            return Ok(self.clone());
        }
        if !xs.derives_from(Xs::UntypedAtomic) {
            todo!("We can only cast to atomic types right now")
        }
        if self.schema_type() == xs {
            return Ok(self.clone());
        }
        match xs {
            Xs::String => Ok(self.cast_to_string()),
            Xs::UntypedAtomic => Ok(self.cast_to_untyped_atomic()),
            Xs::Boolean => self.cast_to_boolean(),
            Xs::Decimal => self.cast_to_decimal(),
            Xs::Integer => self.cast_to_integer(),
            Xs::Long => self.cast_to_long(),
            Xs::Int => self.cast_to_int(),
            Xs::Short => self.cast_to_short(),
            Xs::Byte => self.cast_to_byte(),
            Xs::UnsignedLong => self.cast_to_unsigned_long(),
            Xs::UnsignedInt => self.cast_to_unsigned_int(),
            Xs::UnsignedShort => self.cast_to_unsigned_short(),
            Xs::UnsignedByte => self.cast_to_unsigned_byte(),
            Xs::Float => self.cast_to_float(),
            Xs::Double => self.cast_to_double(),
            Xs::NonPositiveInteger => self.cast_to_non_positive_integer(),
            Xs::NegativeInteger => self.cast_to_negative_integer(),
            Xs::NonNegativeInteger => self.cast_to_non_negative_integer(),
            Xs::PositiveInteger => self.cast_to_positive_integer(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn cast_to_schema_type_of(
        &self,
        other: &atomic::Atomic,
    ) -> error::Result<atomic::Atomic> {
        self.cast_to_schema_type(other.schema_type())
    }

    // if a derives from b, cast to b, otherwise vice versa
    pub(crate) fn cast_to_same_schema_type(
        &self,
        other: &atomic::Atomic,
    ) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
        if self.derives_from(other) {
            let a = self.cast_to_schema_type_of(other)?;
            Ok((a, other.clone()))
        } else if other.derives_from(self) {
            let b = other.cast_to_schema_type_of(self)?;
            Ok((self.clone(), b))
        } else {
            Err(error::Error::Type)
        }
    }

    pub(crate) fn cast_to_string(&self) -> atomic::Atomic {
        atomic::Atomic::String(Rc::new(self.to_canonical()))
    }

    pub(crate) fn cast_to_untyped_atomic(&self) -> atomic::Atomic {
        atomic::Atomic::Untyped(Rc::new(self.to_canonical()))
    }

    pub(crate) fn cast_to_float(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Float(_) => Ok(self.clone()),
            // TODO: this should implement the rule in 19.1.2.1
            atomic::Atomic::Double(OrderedFloat(d)) => {
                // https://www.w3.org/TR/xpath-functions-31/#casting-to-numerics
                // specifies a complex rule that involves the ranges for e of
                // -149 to 104. As far as I can tell Rust uses a different
                // range, -125 to 128. Both refer to IEEE 754-2008, so I'm
                // confused. For now I'm keeping the simple implementation
                // below, which may not behave exactly as per the conversion
                // rules at the extremes of the ranges.
                Ok(atomic::Atomic::Float(OrderedFloat(*d as f32)))
            }
            atomic::Atomic::Decimal(_)
            | atomic::Atomic::Integer(_)
            | atomic::Atomic::NonPositiveInteger(_)
            | atomic::Atomic::NegativeInteger(_)
            | atomic::Atomic::NonNegativeInteger(_)
            | atomic::Atomic::PositiveInteger(_)
            | atomic::Atomic::Long(_)
            | atomic::Atomic::Int(_)
            | atomic::Atomic::Short(_)
            | atomic::Atomic::Byte(_)
            | atomic::Atomic::UnsignedLong(_)
            | atomic::Atomic::UnsignedInt(_)
            | atomic::Atomic::UnsignedShort(_)
            | atomic::Atomic::UnsignedByte(_) => Self::parse_atomic::<f32>(&self.to_canonical()),
            // TODO: any type of integer needs to cast to string first,
            // then to that from float
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Float(OrderedFloat(1.0)))
                } else {
                    Ok(atomic::Atomic::Float(OrderedFloat(0.0)))
                }
            }
            atomic::Atomic::String(s) => Self::parse_atomic::<f32>(s),
            atomic::Atomic::Untyped(s) => Self::parse_atomic::<f32>(s),
        }
    }

    pub(crate) fn cast_to_double(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Double(_) => Ok(self.clone()),
            atomic::Atomic::Float(OrderedFloat(f)) => {
                Ok(atomic::Atomic::Double(OrderedFloat(*f as f64)))
            }
            atomic::Atomic::Decimal(_)
            | atomic::Atomic::Integer(_)
            | atomic::Atomic::NonPositiveInteger(_)
            | atomic::Atomic::NegativeInteger(_)
            | atomic::Atomic::NonNegativeInteger(_)
            | atomic::Atomic::PositiveInteger(_)
            | atomic::Atomic::Long(_)
            | atomic::Atomic::Int(_)
            | atomic::Atomic::Short(_)
            | atomic::Atomic::Byte(_)
            | atomic::Atomic::UnsignedLong(_)
            | atomic::Atomic::UnsignedInt(_)
            | atomic::Atomic::UnsignedShort(_)
            | atomic::Atomic::UnsignedByte(_) => Self::parse_atomic::<f64>(&self.to_canonical()),
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Double(OrderedFloat(1.0)))
                } else {
                    Ok(atomic::Atomic::Double(OrderedFloat(0.0)))
                }
            }
            atomic::Atomic::String(s) => Self::parse_atomic::<f64>(s),
            atomic::Atomic::Untyped(s) => Self::parse_atomic::<f64>(s),
        }
    }

    pub(crate) fn cast_to_decimal(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Decimal(_) => Ok(self.clone()),
            atomic::Atomic::Integer(i)
            | atomic::Atomic::NonPositiveInteger(i)
            | atomic::Atomic::NegativeInteger(i)
            | atomic::Atomic::PositiveInteger(i)
            | atomic::Atomic::NonNegativeInteger(i) => Ok(atomic::Atomic::Decimal(
                // rust decimal doesn't support arbitrary precision integers,
                // so we fail some conversions
                Decimal::try_from_i128_with_scale(
                    // if this is bigger than an i128, it certainly can't be
                    // an integer
                    i.as_ref().try_into().map_err(|_| error::Error::Overflow)?,
                    0,
                )
                .map_err(|_| error::Error::Overflow)?,
            )),
            atomic::Atomic::Long(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::Int(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::Short(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::Byte(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::UnsignedLong(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::UnsignedInt(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::UnsignedShort(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::UnsignedByte(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
            atomic::Atomic::Float(OrderedFloat(f)) => {
                if f.is_nan() || f.is_infinite() {
                    return Err(error::Error::FOCA0002);
                }

                Ok(atomic::Atomic::Decimal(
                    Decimal::try_from(*f).map_err(|_| error::Error::FOCA0001)?,
                ))
            }
            atomic::Atomic::Double(OrderedFloat(f)) => {
                if f.is_nan() || f.is_infinite() {
                    return Err(error::Error::FOCA0002);
                }

                Ok(atomic::Atomic::Decimal(
                    Decimal::try_from(*f).map_err(|_| error::Error::FOCA0001)?,
                ))
            }
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Decimal(Decimal::from(1)))
                } else {
                    Ok(atomic::Atomic::Decimal(Decimal::from(0)))
                }
            }
            atomic::Atomic::String(s) => Self::parse_atomic::<Decimal>(s),
            atomic::Atomic::Untyped(s) => Self::parse_atomic::<Decimal>(s),
        }
    }

    pub(crate) fn cast_to_integer(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::Integer(Rc::new(
            self.cast_to_integer_value::<IBig>()?,
        )))
    }

    pub(crate) fn cast_to_long(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::Long(self.cast_to_integer_value::<i64>()?))
    }

    pub(crate) fn cast_to_int(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::Int(self.cast_to_integer_value::<i32>()?))
    }

    pub(crate) fn cast_to_short(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::Short(self.cast_to_integer_value::<i16>()?))
    }

    pub(crate) fn cast_to_byte(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::Byte(self.cast_to_integer_value::<i8>()?))
    }

    pub(crate) fn cast_to_unsigned_long(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::UnsignedLong(
            self.cast_to_integer_value::<u64>()?,
        ))
    }

    pub(crate) fn cast_to_unsigned_int(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::UnsignedInt(
            self.cast_to_integer_value::<u32>()?,
        ))
    }

    pub(crate) fn cast_to_unsigned_short(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::UnsignedShort(
            self.cast_to_integer_value::<u16>()?,
        ))
    }

    pub(crate) fn cast_to_unsigned_byte(&self) -> error::Result<atomic::Atomic> {
        Ok(atomic::Atomic::UnsignedByte(
            self.cast_to_integer_value::<u8>()?,
        ))
    }

    pub(crate) fn cast_to_non_positive_integer(&self) -> error::Result<atomic::Atomic> {
        let i = self.cast_to_integer_value::<IBig>()?;
        if i > ibig!(0) {
            return Err(error::Error::FOCA0003);
        }
        Ok(atomic::Atomic::NonPositiveInteger(Rc::new(
            self.cast_to_integer_value::<IBig>()?,
        )))
    }

    pub(crate) fn cast_to_negative_integer(&self) -> error::Result<atomic::Atomic> {
        let i = self.cast_to_integer_value::<IBig>()?;
        if i >= ibig!(0) {
            return Err(error::Error::FOCA0003);
        }
        Ok(atomic::Atomic::NegativeInteger(Rc::new(
            self.cast_to_integer_value::<IBig>()?,
        )))
    }

    pub(crate) fn cast_to_non_negative_integer(&self) -> error::Result<atomic::Atomic> {
        let i = self.cast_to_integer_value::<IBig>()?;
        if i < ibig!(0) {
            return Err(error::Error::FOCA0003);
        }
        Ok(atomic::Atomic::NonNegativeInteger(Rc::new(
            self.cast_to_integer_value::<IBig>()?,
        )))
    }

    pub(crate) fn cast_to_positive_integer(&self) -> error::Result<atomic::Atomic> {
        let i = self.cast_to_integer_value::<IBig>()?;
        if i <= ibig!(0) {
            return Err(error::Error::FOCA0003);
        }
        Ok(atomic::Atomic::PositiveInteger(Rc::new(
            self.cast_to_integer_value::<IBig>()?,
        )))
    }

    pub(crate) fn cast_to_integer_value<V>(&self) -> error::Result<V>
    where
        V: TryFrom<IBig>
            + TryFrom<i128>
            + TryFrom<i64>
            + TryFrom<i32>
            + TryFrom<i16>
            + TryFrom<i8>
            + TryFrom<u64>
            + TryFrom<u32>
            + TryFrom<u16>
            + TryFrom<u8>,
        Parsed<V>: FromStr<Err = error::Error>,
    {
        match self {
            atomic::Atomic::Integer(i)
            | atomic::Atomic::NonPositiveInteger(i)
            | atomic::Atomic::NegativeInteger(i)
            | atomic::Atomic::NonNegativeInteger(i)
            | atomic::Atomic::PositiveInteger(i) => {
                let i: V = i
                    .as_ref()
                    .clone()
                    .try_into()
                    .map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Long(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Int(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Short(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Byte(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::UnsignedLong(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::UnsignedInt(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::UnsignedShort(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::UnsignedByte(i) => {
                let i: V = (*i).try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Decimal(d) => {
                // we first go to a i128; this accomodates the largest Decimal
                let i: i128 = d.trunc().try_into().map_err(|_| error::Error::FOCA0003)?;
                // then we convert this into the target integer type
                let i: V = i.try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Float(OrderedFloat(f)) => {
                if f.is_nan() | f.is_infinite() {
                    return Err(error::Error::FOCA0002);
                }
                // we first go to a decimal. Any larger number we won't be able to
                // express, even though bigint strictly speaking could handle it.
                // But converting a float to a bigint directly isn't possible.
                let d: Decimal = f.trunc().try_into().map_err(|_| error::Error::FOCA0003)?;
                let i: i128 = d.try_into().map_err(|_| error::Error::FOCA0003)?;
                let i: V = i.try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Double(OrderedFloat(d)) => {
                if d.is_nan() | d.is_infinite() {
                    return Err(error::Error::FOCA0002);
                }
                // we first go to a decimal. Any larger number we won't be able to
                // express, even though bigint strictly speaking could handle it.
                // But converting a float to a bigint directly isn't possible.
                let d: Decimal = d.trunc().try_into().map_err(|_| error::Error::FOCA0003)?;
                let i: i128 = d.try_into().map_err(|_| error::Error::FOCA0003)?;
                let i: V = i.try_into().map_err(|_| error::Error::FOCA0003)?;
                Ok(i)
            }
            atomic::Atomic::Boolean(b) => {
                let v: V = if *b {
                    1.try_into().map_err(|_| error::Error::FOCA0003)?
                } else {
                    0.try_into().map_err(|_| error::Error::FOCA0003)?
                };
                Ok(v)
            }
            atomic::Atomic::String(s) => Ok(s.parse::<Parsed<V>>()?.into_inner()),
            atomic::Atomic::Untyped(s) => Ok(s.parse::<Parsed<V>>()?.into_inner()),
        }
    }

    pub(crate) fn cast_to_boolean(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Boolean(_) => Ok(self.clone()),
            atomic::Atomic::Float(f) => Ok(atomic::Atomic::Boolean(!(f.is_nan() || f.is_zero()))),
            atomic::Atomic::Double(d) => Ok(atomic::Atomic::Boolean(!(d.is_nan() || d.is_zero()))),
            atomic::Atomic::Decimal(d) => Ok(atomic::Atomic::Boolean(!d.is_zero())),
            atomic::Atomic::Integer(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Long(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Int(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Short(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Byte(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedLong(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedInt(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedShort(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedByte(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::NonNegativeInteger(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::PositiveInteger(_) => Ok(true.into()),
            atomic::Atomic::NonPositiveInteger(i) => Ok(atomic::Atomic::Boolean(i.is_zero())),
            atomic::Atomic::NegativeInteger(_) => Ok(true.into()),
            atomic::Atomic::String(s) => Self::parse_atomic::<bool>(s),
            atomic::Atomic::Untyped(s) => Self::parse_atomic::<bool>(s),
        }
    }
}

// shared casting logic for binary operations; both comparison and arithmetic use
// this
pub(crate) fn cast_numeric_binary<F>(
    a: atomic::Atomic,
    b: atomic::Atomic,
    non_numeric: F,
) -> error::Result<(atomic::Atomic, atomic::Atomic)>
where
    F: Fn(atomic::Atomic, atomic::Atomic) -> error::Result<(atomic::Atomic, atomic::Atomic)>,
{
    let a_schema_type = a.schema_type();
    let b_schema_type = b.schema_type();

    // early return for case where the concrete types are already correct
    use Xs::*;
    match (a_schema_type, b_schema_type) {
        (Float, Float) | (Double, Double) | (Decimal, Decimal) | (Integer, Integer) => {
            return Ok((a, b))
        }
        _ => {}
    }

    // otherwise we need to do some casting
    let a_numeric_type = a_schema_type.base_numeric_type();
    let b_numeric_type = b_schema_type.base_numeric_type();

    match (a_numeric_type, b_numeric_type) {
        (None, None) | (_, None) | (None, _) => non_numeric(a, b),

        (Some(a_numeric_type), Some(b_numeric_type)) => {
            // this is in terms of the base numeric schema type
            use BaseNumericType::*;
            match (a_numeric_type, b_numeric_type) {
                // 5b: xs:decimal & xs:float -> cast decimal to float
                (Decimal, Float) | (Integer, Float) | (Float, Decimal) | (Float, Integer) => {
                    Ok((a.cast_to_float()?, b.cast_to_float()?))
                }
                // 5c: xs:decimal & xs:double -> cast decimal to double
                (Decimal, Double) | (Integer, Double) | (Double, Decimal) | (Double, Integer) => {
                    Ok((a.cast_to_double()?, b.cast_to_double()?))
                }
                // 5c: xs:float & xs:double -> cast float to double
                (Float, Double) | (Double, Float) => Ok((a.cast_to_double()?, b.cast_to_double()?)),
                // both are floats
                (Float, Float) => Ok((a.cast_to_float()?, b.cast_to_float()?)),
                // both are doubles
                (Double, Double) => Ok((a.cast_to_double()?, b.cast_to_double()?)),
                // both are decimals
                (Decimal, Decimal) | (Decimal, Integer) | (Integer, Decimal) => {
                    Ok((a.cast_to_decimal()?, b.cast_to_decimal()?))
                }
                // both are integers of some type
                (Integer, Integer) => Ok((a.cast_to_integer()?, b.cast_to_integer()?)),
            }
        }
    }
}

pub(crate) struct Parsed<V>(V);

impl<V> Parsed<V> {
    fn into_inner(self) -> V {
        self.0
    }
}

impl FromStr for Parsed<Decimal> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_decimal(s)?))
    }
}

impl FromStr for Parsed<IBig> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(s.parse().map_err(|_| error::Error::FOCA0003)?))
    }
}

impl FromStr for Parsed<i64> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<i32> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<i16> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<i8> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<u64> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<u32> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<u16> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<u8> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_integer_number(s)?))
    }
}

impl FromStr for Parsed<f64> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_double(s)?))
    }
}

impl FromStr for Parsed<f32> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_float(s)?))
    }
}

impl FromStr for Parsed<bool> {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Parsed(atomic::Atomic::parse_boolean(s)?))
    }
}

struct WrappedIBig(IBig);

fn canonical_float<F>(f: F) -> String
where
    F: Float
        + TryInto<Decimal, Error = rust_decimal::Error>
        + lexical::ToLexicalWithOptions<Options = lexical::WriteFloatOptions>
        + num::Signed,
{
    // https://www.w3.org/TR/xpath-functions-31/#casting-to-string
    // If SV has an absolute value that is greater than or equal to
    // 0.000001 (one millionth) and less than 1000000 (one
    // million), then the value is converted to an xs:decimal and
    // the resulting xs:decimal is converted to an xs:string
    let abs_f = f.abs();
    let minimum: F = num::cast(0.000001).unwrap();
    let maximum: F = num::cast(1000000.0).unwrap();
    if abs_f >= minimum && abs_f < maximum {
        // TODO: is this the right conversion?
        let d: Decimal = f.try_into().unwrap();
        atomic::Atomic::Decimal(d).to_canonical()
    } else {
        if f.is_zero() {
            if f.is_negative() {
                return "-0".to_string();
            } else {
                return "0".to_string();
            }
        }
        let options = lexical::WriteFloatOptionsBuilder::new()
            .exponent(b'E')
            .inf_string(Some(b"INF"))
            .build()
            .unwrap();
        lexical::to_string_with_options::<_, { lexical::format::XML }>(f, &options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ibig::ibig;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_decimal() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<Decimal>("1.0").unwrap(),
            atomic::Atomic::Decimal(dec!(1.0))
        );
    }

    #[test]
    fn test_parse_decimal_no_underscore() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<Decimal>("1_000.0"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<IBig>("1").unwrap(),
            atomic::Atomic::Integer(ibig!(1).into())
        );
    }

    #[test]
    fn test_parse_long() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<i64>("1").unwrap(),
            atomic::Atomic::Long(1)
        );
    }

    #[test]
    fn test_parse_integer_no_underscore() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<i64>("1_000"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_parse_double() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("1.0").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0))
        );
    }

    #[test]
    fn test_parse_double_exponent() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("1.0e10").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0e10))
        );
    }

    #[test]
    fn test_parse_double_exponent_capital() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("1.0E10").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0e10))
        );
    }

    #[test]
    fn test_parse_double_inf() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("INF").unwrap(),
            atomic::Atomic::Double(OrderedFloat(f64::INFINITY))
        );
    }

    #[test]
    fn test_parse_double_minus_inf() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("-INF").unwrap(),
            atomic::Atomic::Double(OrderedFloat(-f64::INFINITY))
        );
    }

    #[test]
    fn test_parse_double_nan() {
        let a = atomic::Atomic::parse_atomic::<f64>("NaN").unwrap();
        match a {
            atomic::Atomic::Double(OrderedFloat(f)) => assert!(f.is_nan()),
            _ => panic!("Expected double"),
        }
    }

    #[test]
    fn test_parse_double_invalid_nan() {
        assert_eq!(
            atomic::Atomic::parse_atomic::<f64>("NAN"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_canonical_decimal_is_integer() {
        assert_eq!(atomic::Atomic::Decimal(dec!(1.0)).to_canonical(), "1");
    }

    #[test]
    fn test_canonical_decimal_is_decimal() {
        assert_eq!(atomic::Atomic::Decimal(dec!(1.5)).to_canonical(), "1.5");
    }

    #[test]
    fn test_canonical_decimal_no_trailing_zeroes() {
        assert_eq!(atomic::Atomic::Decimal(dec!(1.50)).to_canonical(), "1.5");
    }

    #[test]
    fn test_canonical_decimal_no_leading_zeroes() {
        assert_eq!(atomic::Atomic::Decimal(dec!(01.50)).to_canonical(), "1.5");
    }

    #[test]
    fn test_canonical_decimal_single_leading_zero() {
        assert_eq!(atomic::Atomic::Decimal(dec!(0.50)).to_canonical(), "0.5");
    }

    #[test]
    fn test_canonical_integer() {
        assert_eq!(
            atomic::Atomic::Integer(ibig!(15).into()).to_canonical(),
            "15"
        );
    }

    #[test]
    fn test_canonical_float_formatted_as_integer() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(15.0)).to_canonical(),
            "15"
        );
    }

    #[test]
    fn test_canonical_float_formatted_as_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(15.5)).to_canonical(),
            "15.5"
        );
    }

    #[test]
    fn test_canonical_float_formatted_as_float_big() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(1500000000000000f32)).to_canonical(),
            "1.5E15"
        );
    }

    #[test]
    fn test_canonical_formatted_as_float_small() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(0.000000000000001f32)).to_canonical(),
            "1.0E-15"
        );
    }

    #[test]
    fn test_canonical_float_zero() {
        assert_eq!(atomic::Atomic::Float(OrderedFloat(0.0)).to_canonical(), "0");
    }

    #[test]
    fn test_canonical_float_minus_zero() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(-0.0)).to_canonical(),
            "-0"
        );
    }

    #[test]
    fn test_canonical_float_inf() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY)).to_canonical(),
            "INF"
        );
    }

    #[test]
    fn test_canonical_double_formatted_as_decimal() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(15.5)).to_canonical(),
            "15.5"
        );
    }

    #[test]
    fn test_cast_double_to_float_inf() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f64::INFINITY))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY))
        );
    }

    #[test]
    fn test_cast_double_to_float_negative_inf() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f64::NEG_INFINITY))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::NEG_INFINITY))
        );
    }

    #[test]
    fn test_cast_double_to_float_nan() {
        let a = atomic::Atomic::Double(OrderedFloat(f64::NAN))
            .cast_to_float()
            .unwrap();
        match a {
            atomic::Atomic::Float(OrderedFloat(f)) => assert!(f.is_nan()),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_cast_double_to_float_zero() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(0.0))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(0.0))
        );
    }

    #[test]
    fn test_cast_double_to_float_negative_zero() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(-0.0))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(-0.0))
        );
    }

    #[test]
    fn test_cast_double_to_float_too_big() {
        // TODO: are the boundaries exactly as in spec?
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f32::MAX as f64 * 1.1))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY))
        );
    }

    #[test]
    fn test_cast_double_to_float_too_small() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(1e-150))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(0.0))
        );
    }

    #[test]
    fn test_cast_double_to_float_not_too_small() {
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(1e-5))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(1e-5))
        );
    }

    #[test]
    fn test_cast_int_to_decimal() {
        assert_eq!(
            atomic::Atomic::Int(15).cast_to_decimal().unwrap(),
            atomic::Atomic::Decimal(dec!(15))
        );
    }

    #[test]
    fn test_cast_float_to_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(15.5))
                .cast_to_decimal()
                .unwrap(),
            atomic::Atomic::Decimal(dec!(15.5))
        );
    }

    #[test]
    fn test_cast_inf_float_to_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(f32::INFINITY)).cast_to_decimal(),
            Err(error::Error::FOCA0002)
        );
    }

    #[test]
    fn test_cast_huge_float_to_decimal() {
        assert_eq!(
            atomic::Atomic::Float(OrderedFloat(1.0e30)).cast_to_decimal(),
            Err(error::Error::FOCA0001)
        );
    }

    #[test]
    fn test_cast_short_to_short() {
        assert_eq!(
            atomic::Atomic::Short(15).cast_to_short().unwrap(),
            atomic::Atomic::Short(15)
        );
    }
}
