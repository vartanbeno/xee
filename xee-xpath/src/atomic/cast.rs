use std::rc::Rc;

use ordered_float::OrderedFloat;

use crate::atomic;
use crate::error;

impl atomic::Atomic {
    pub(crate) fn parse_decimal(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: parse decimal, but reject any string with _ in it as invalid
        todo!();
    }

    // from an atomic type to a canonical representation as a string
    pub(crate) fn canonical_representation(&self) -> error::Result<String> {
        todo!();
    }

    pub(crate) fn cast_to_xs_string(&self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::String(_) => Ok(self.clone()),
            atomic::Atomic::Untyped(s) => Ok(atomic::Atomic::String(s.clone())),
            atomic::Atomic::Boolean(b) => {
                if *b {
                    Ok(atomic::Atomic::String(Rc::new("true".to_string())))
                } else {
                    Ok(atomic::Atomic::String(Rc::new("false".to_string())))
                }
            }
            atomic::Atomic::Decimal(d) => {
                if d.is_integer() {
                    let i: i64 = (*d).try_into().map_err(|_| error::Error::FOCA0003)?;
                    Ok(atomic::Atomic::String(Rc::new(i.to_string())))
                } else {
                    // TODO: is this really the caonical lexical representation?
                    Ok(atomic::Atomic::String(Rc::new(d.to_string())))
                }
            }
            atomic::Atomic::Integer(i) => Ok(atomic::Atomic::String(Rc::new(i.to_string()))),
            atomic::Atomic::Int(i) => Ok(atomic::Atomic::String(Rc::new(i.to_string()))),
            atomic::Atomic::Short(s) => Ok(atomic::Atomic::String(Rc::new(s.to_string()))),
            atomic::Atomic::Byte(b) => Ok(atomic::Atomic::String(Rc::new(b.to_string()))),
            atomic::Atomic::UnsignedLong(u) => Ok(atomic::Atomic::String(Rc::new(u.to_string()))),
            atomic::Atomic::UnsignedInt(u) => Ok(atomic::Atomic::String(Rc::new(u.to_string()))),
            atomic::Atomic::UnsignedShort(u) => Ok(atomic::Atomic::String(Rc::new(u.to_string()))),
            atomic::Atomic::UnsignedByte(u) => Ok(atomic::Atomic::String(Rc::new(u.to_string()))),
            atomic::Atomic::Float(f) => Ok(atomic::Atomic::String(Rc::new(f.to_string()))),
            atomic::Atomic::Double(d) => Ok(atomic::Atomic::String(Rc::new(d.to_string()))),
            atomic::Atomic::Absent => {
                // TODO: remove absent from atomic
                panic!("Absent atomics should not be cast to string")
            }
        }
    }

    pub(crate) fn cast_to_untyped_atomic(&self) -> error::Result<atomic::Atomic> {
        let s = self.cast_to_xs_string()?;
        if let atomic::Atomic::String(s) = s {
            Ok(atomic::Atomic::Untyped(s))
        } else {
            unreachable!("cast_to_xs_string should always return a string")
        }
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

    use rust_decimal::prelude::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_decimal_string_parse() {
        let d: Decimal = "1.0".parse().unwrap();
        assert_eq!(d, dec!(1.0));

        let d2: Decimal = "100_000".parse().unwrap();
        assert_eq!(d2, dec!(100_000));
        assert_eq!(d2.to_string(), "100000");
    }
}
