use num_traits::Float;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::rc::Rc;

use crate::atomic;
use crate::error;

impl atomic::Atomic {
    pub(crate) fn parse_decimal(s: &str) -> error::Result<atomic::Atomic> {
        if s.contains('_') {
            return Err(error::Error::FORG0001);
        }
        s.parse::<rust_decimal::Decimal>()
            .map(atomic::Atomic::Decimal)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_integer(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<i64, _>(s)
            .map(atomic::Atomic::Integer)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_int(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<i32, _>(s)
            .map(atomic::Atomic::Int)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_short(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<i16, _>(s)
            .map(atomic::Atomic::Short)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_byte(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<i8, _>(s)
            .map(atomic::Atomic::Byte)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_unsigned_long(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<u64, _>(s)
            .map(atomic::Atomic::UnsignedLong)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_unsigned_int(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<u32, _>(s)
            .map(atomic::Atomic::UnsignedInt)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_unsigned_short(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<u16, _>(s)
            .map(atomic::Atomic::UnsignedShort)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_unsigned_byte(s: &str) -> error::Result<atomic::Atomic> {
        lexical::parse::<u8, _>(s)
            .map(atomic::Atomic::UnsignedByte)
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_boolean(s: &str) -> error::Result<atomic::Atomic> {
        match s {
            "true" => Ok(atomic::Atomic::Boolean(true)),
            "false" => Ok(atomic::Atomic::Boolean(false)),
            _ => Err(error::Error::FORG0001),
        }
    }

    // we can't use lexical::parse_float_options::XML as it doesn't allow INF
    // which is allowed by the XML Schema spec

    pub(crate) fn parse_float(s: &str) -> error::Result<atomic::Atomic> {
        let options = lexical::ParseFloatOptionsBuilder::new()
            .inf_string(Some(b"INF"))
            .build()
            .unwrap();
        lexical::parse_with_options::<f32, _, { lexical::format::XML }>(s, &options)
            .map(|f| atomic::Atomic::Float(OrderedFloat(f)))
            .map_err(|_| error::Error::FORG0001)
    }

    pub(crate) fn parse_double(s: &str) -> error::Result<atomic::Atomic> {
        let options = lexical::ParseFloatOptionsBuilder::new()
            .inf_string(Some(b"INF"))
            .build()
            .unwrap();
        lexical::parse_with_options::<f64, _, { lexical::format::XML }>(s, &options)
            .map(|f| atomic::Atomic::Double(OrderedFloat(f)))
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
            atomic::Atomic::Int(i) => i.to_string(),
            atomic::Atomic::Short(i) => i.to_string(),
            atomic::Atomic::Byte(i) => i.to_string(),
            atomic::Atomic::UnsignedLong(i) => i.to_string(),
            atomic::Atomic::UnsignedInt(i) => i.to_string(),
            atomic::Atomic::UnsignedShort(i) => i.to_string(),
            atomic::Atomic::UnsignedByte(i) => i.to_string(),
            atomic::Atomic::Float(OrderedFloat(f)) => canonical_float(*f),
            atomic::Atomic::Double(OrderedFloat(f)) => canonical_float(*f),
            _ => {
                todo!()
            }
        }
    }

    pub(crate) fn cast_to_xs_string(&self) -> atomic::Atomic {
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
            | atomic::Atomic::Int(_)
            | atomic::Atomic::Short(_)
            | atomic::Atomic::Byte(_)
            | atomic::Atomic::UnsignedLong(_)
            | atomic::Atomic::UnsignedInt(_)
            | atomic::Atomic::UnsignedShort(_)
            | atomic::Atomic::UnsignedByte(_) => Self::parse_float(&self.to_canonical()),
            // TODO: any type of integer needs to cast to string first,
            // then to that from float
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Float(OrderedFloat(1.0)))
                } else {
                    Ok(atomic::Atomic::Float(OrderedFloat(0.0)))
                }
            }
            atomic::Atomic::String(s) => Self::parse_float(s),
            atomic::Atomic::Untyped(s) => Self::parse_float(s),
            _ => {
                panic!("absent not supported")
            }
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
            | atomic::Atomic::Int(_)
            | atomic::Atomic::Short(_)
            | atomic::Atomic::Byte(_)
            | atomic::Atomic::UnsignedLong(_)
            | atomic::Atomic::UnsignedInt(_)
            | atomic::Atomic::UnsignedShort(_)
            | atomic::Atomic::UnsignedByte(_) => Self::parse_double(&self.to_canonical()),
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Double(OrderedFloat(1.0)))
                } else {
                    Ok(atomic::Atomic::Double(OrderedFloat(0.0)))
                }
            }
            atomic::Atomic::String(s) => Self::parse_double(s),
            atomic::Atomic::Untyped(s) => Self::parse_double(s),
            _ => {
                panic!("absent not supported")
            }
        }
    }

    pub(crate) fn cast_to_decimal(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Decimal(_) => Ok(self.clone()),
            atomic::Atomic::Integer(i) => Ok(atomic::Atomic::Decimal(Decimal::from(*i))),
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
            atomic::Atomic::String(s) => Self::parse_decimal(s),
            atomic::Atomic::Untyped(s) => Self::parse_decimal(s),
            _ => {
                panic!("absent not supported")
            }
        }
    }

    pub(crate) fn cast_to_integer(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Integer(_)
            | atomic::Atomic::Int(_)
            | atomic::Atomic::Short(_)
            | atomic::Atomic::Byte(_)
            | atomic::Atomic::UnsignedLong(_)
            | atomic::Atomic::UnsignedInt(_)
            | atomic::Atomic::UnsignedShort(_)
            | atomic::Atomic::UnsignedByte(_) => Ok(self.clone()),
            atomic::Atomic::Decimal(d) => d
                .trunc()
                .try_into()
                .map(atomic::Atomic::Integer)
                .map_err(|_| error::Error::FOCA0003),
            atomic::Atomic::Float(OrderedFloat(f)) => {
                if f.is_nan() | f.is_infinite() {
                    return Err(error::Error::FOCA0002);
                }
                let i: i64 = f.trunc().to_i64().ok_or(error::Error::FOCA0003)?;
                Ok(atomic::Atomic::Integer(i))
            }
            atomic::Atomic::Double(OrderedFloat(d)) => {
                if d.is_nan() | d.is_infinite() {
                    return Err(error::Error::FOCA0002);
                }
                let i: i64 = d.trunc().to_i64().ok_or(error::Error::FOCA0003)?;
                Ok(atomic::Atomic::Integer(i))
            }
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Integer(1))
                } else {
                    Ok(atomic::Atomic::Integer(0))
                }
            }
            atomic::Atomic::String(s) => Self::parse_integer(s),
            atomic::Atomic::Untyped(s) => Self::parse_integer(s),
            atomic::Atomic::Absent => {
                panic!("absent not supported")
            }
        }
    }

    pub(crate) fn cast_to_boolean(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Boolean(_) => Ok(self.clone()),
            atomic::Atomic::Float(f) => Ok(atomic::Atomic::Boolean(!(f.is_nan() || f.is_zero()))),
            atomic::Atomic::Double(d) => Ok(atomic::Atomic::Boolean(!(d.is_nan() || d.is_zero()))),
            atomic::Atomic::Decimal(d) => Ok(atomic::Atomic::Boolean(!d.is_zero())),
            atomic::Atomic::Integer(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Int(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Short(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::Byte(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedLong(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedInt(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedShort(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::UnsignedByte(i) => Ok(atomic::Atomic::Boolean(!i.is_zero())),
            atomic::Atomic::String(s) => Self::parse_boolean(s),
            atomic::Atomic::Untyped(s) => Self::parse_boolean(s),
            _ => {
                panic!("absent not supported")
            }
        }
    }
}

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

    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_decimal() {
        assert_eq!(
            atomic::Atomic::parse_decimal("1.0").unwrap(),
            atomic::Atomic::Decimal(dec!(1.0))
        );
    }

    #[test]
    fn test_parse_decimal_no_underscore() {
        assert_eq!(
            atomic::Atomic::parse_decimal("1_000.0"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            atomic::Atomic::parse_integer("1").unwrap(),
            atomic::Atomic::Integer(1)
        );
    }

    #[test]
    fn test_parse_integer_no_underscore() {
        assert_eq!(
            atomic::Atomic::parse_integer("1_000"),
            Err(error::Error::FORG0001)
        );
    }

    #[test]
    fn test_parse_double() {
        assert_eq!(
            atomic::Atomic::parse_double("1.0").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0))
        );
    }

    #[test]
    fn test_parse_double_exponent() {
        assert_eq!(
            atomic::Atomic::parse_double("1.0e10").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0e10))
        );
    }

    #[test]
    fn test_parse_double_exponent_capital() {
        assert_eq!(
            atomic::Atomic::parse_double("1.0E10").unwrap(),
            atomic::Atomic::Double(OrderedFloat(1.0e10))
        );
    }

    #[test]
    fn test_parse_double_inf() {
        assert_eq!(
            atomic::Atomic::parse_double("INF").unwrap(),
            atomic::Atomic::Double(OrderedFloat(f64::INFINITY))
        );
    }

    #[test]
    fn test_parse_double_minus_inf() {
        assert_eq!(
            atomic::Atomic::parse_double("-INF").unwrap(),
            atomic::Atomic::Double(OrderedFloat(-f64::INFINITY))
        );
    }

    #[test]
    fn test_parse_double_nan() {
        assert_eq!(
            atomic::Atomic::parse_double("NaN").unwrap(),
            atomic::Atomic::Double(OrderedFloat(f64::NAN))
        );
    }

    #[test]
    fn test_parse_double_invalid_nan() {
        assert_eq!(
            atomic::Atomic::parse_double("NAN"),
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
        assert_eq!(atomic::Atomic::Integer(15).to_canonical(), "15");
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
        assert_eq!(
            atomic::Atomic::Double(OrderedFloat(f64::NAN))
                .cast_to_float()
                .unwrap(),
            atomic::Atomic::Float(OrderedFloat(f32::NAN))
        );
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
}
