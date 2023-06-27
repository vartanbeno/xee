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
            atomic::Atomic::Float(OrderedFloat(f)) => {
                // https://www.w3.org/TR/xpath-functions-31/#casting-to-string
                // If SV has an absolute value that is greater than or equal to
                // 0.000001 (one millionth) and less than 1000000 (one
                // million), then the value is converted to an xs:decimal and
                // the resulting xs:decimal is converted to an xs:string
                let abs_f = f.abs();
                if (0.000001..1000000f32).contains(&abs_f) {
                    // TODO: is this the right conversion?
                    let d: Decimal = (*f).try_into().unwrap();
                    atomic::Atomic::Decimal(d).to_canonical()
                } else {
                    if (*f).is_zero() {
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
                    lexical::to_string_with_options::<_, { lexical::format::XML }>(*f, &options)
                }
            }
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
            // https://www.w3.org/TR/xpath-functions-31/#casting-to-numerics
            atomic::Atomic::Double(OrderedFloat(d)) => {
                Ok(atomic::Atomic::Float(OrderedFloat(*d as f32)))
            }
            atomic::Atomic::Decimal(_) => {
                // TODO specification says to cast to string first, then
                // from that to float
                todo!();
            }
            // TODO: any type of integer needs to cast to string first,
            // then to that from float
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::Float(OrderedFloat(1.0)))
                } else {
                    Ok(atomic::Atomic::Float(OrderedFloat(0.0)))
                }
            }
            _ => {
                todo!();
            }
        }
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
}
