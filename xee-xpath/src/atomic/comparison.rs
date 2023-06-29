use num_traits::{Float, PrimInt};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use xee_schema_type::Xs;

use crate::error;

use super::atomic_core as atomic;
use super::cast::cast_to_same;

pub(crate) fn comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<bool>
where
    O: ComparisonOp,
{
    let (a, b) = cast(a, b)?;

    // cast guarantees both atomic types are the same concrete atomic
    Ok(match (a, b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => {
            <O as ComparisonOp>::string(&a, &b)
        }
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => {
            <O as ComparisonOp>::boolean(a, b)
        }
        (atomic::Atomic::Decimal(a), atomic::Atomic::Decimal(b)) => {
            <O as ComparisonOp>::decimal(a, b)
        }
        (atomic::Atomic::Integer(a), atomic::Atomic::Integer(b)) => {
            <O as ComparisonOp>::integer(a, b)
        }
        (atomic::Atomic::Int(a), atomic::Atomic::Int(b)) => <O as ComparisonOp>::integer(a, b),
        (atomic::Atomic::Short(a), atomic::Atomic::Short(b)) => <O as ComparisonOp>::integer(a, b),
        (atomic::Atomic::Byte(a), atomic::Atomic::Byte(b)) => <O as ComparisonOp>::integer(a, b),
        (atomic::Atomic::UnsignedLong(a), atomic::Atomic::UnsignedLong(b)) => {
            <O as ComparisonOp>::integer(a, b)
        }
        (atomic::Atomic::UnsignedInt(a), atomic::Atomic::UnsignedInt(b)) => {
            <O as ComparisonOp>::integer(a, b)
        }
        (atomic::Atomic::UnsignedShort(a), atomic::Atomic::UnsignedShort(b)) => {
            <O as ComparisonOp>::integer(a, b)
        }
        (atomic::Atomic::UnsignedByte(a), atomic::Atomic::UnsignedByte(b)) => {
            <O as ComparisonOp>::integer(a, b)
        }
        (atomic::Atomic::Float(OrderedFloat(a)), atomic::Atomic::Float(OrderedFloat(b))) => {
            <O as ComparisonOp>::float(a, b)
        }
        (atomic::Atomic::Double(OrderedFloat(a)), atomic::Atomic::Double(OrderedFloat(b))) => {
            <O as ComparisonOp>::float(a, b)
        }
        _ => unreachable!("Both the atomics are not the same type or types aren't handled"),
    })
}

fn cast(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
    // 3.7.1 Value Comparisons
    // We start in step 4, as the previous steps have been handled
    // by the caller.

    // 4: If an atomized operand of of type xs:untypedAtomic, it is cast
    // to xs:string
    let a = cast_untyped(a);
    let b = cast_untyped(b);

    match (&a, &b) {
        // 5a: TODO: xs:string and xs:anyURI
        // 5b: xs:decimal & xs:float -> cast decimal to float
        (atomic::Atomic::Decimal(_), atomic::Atomic::Float(_)) => Ok((a.cast_to_float()?, b)),
        (atomic::Atomic::Float(_), atomic::Atomic::Decimal(_)) => Ok((a, b.cast_to_float()?)),
        // 5c: xs:decimal & xs:double -> cast decimal to double
        (atomic::Atomic::Decimal(_), atomic::Atomic::Double(_)) => Ok((a.cast_to_double()?, b)),
        (atomic::Atomic::Double(_), atomic::Atomic::Decimal(_)) => Ok((a, b.cast_to_double()?)),
        // 5c: xs:float & xs:double -> cast float to double
        (atomic::Atomic::Float(_), atomic::Atomic::Double(_)) => Ok((a.cast_to_double()?, b)),
        (atomic::Atomic::Double(_), atomic::Atomic::Float(_)) => Ok((a, b.cast_to_double()?)),

        _ => {
            // decimals and all integer types are considered to be the same type
            // This means some fancy casting
            if a.has_base_schema_type(Xs::Decimal) && b.has_base_schema_type(Xs::Decimal) {
                cast_to_same(a, b)
            } else {
                // if we're the type, we're done
                if a.has_same_schema_type(&b) {
                    Ok((a, b))
                } else {
                    // We're not handling derived non-atomic data types,
                    // which is okay as atomization has taking place already
                    // 5d otherwise, type error
                    Err(error::Error::Type)
                }
            }
        }
    }
}

fn cast_untyped(value: atomic::Atomic) -> atomic::Atomic {
    if let atomic::Atomic::Untyped(s) = value {
        atomic::Atomic::String(s)
    } else {
        value
    }
}

pub(crate) trait ComparisonOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt;
    fn decimal(a: Decimal, b: Decimal) -> bool;
    fn float<F>(a: F, b: F) -> bool
    where
        F: Float;
    fn string(a: &str, b: &str) -> bool;
    fn boolean(a: bool, b: bool) -> bool;
}

pub(crate) struct EqualOp;

impl ComparisonOp for EqualOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt,
    {
        a == b
    }

    fn decimal(a: Decimal, b: Decimal) -> bool {
        a == b
    }

    fn float<F>(a: F, b: F) -> bool
    where
        F: Float,
    {
        a == b
    }

    fn string(a: &str, b: &str) -> bool {
        a == b
    }

    fn boolean(a: bool, b: bool) -> bool {
        a == b
    }
}

pub(crate) struct NotEqualOp;

impl ComparisonOp for NotEqualOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt,
    {
        a != b
    }

    fn decimal(a: Decimal, b: Decimal) -> bool {
        a != b
    }

    fn float<F>(a: F, b: F) -> bool
    where
        F: Float,
    {
        a != b
    }

    fn string(a: &str, b: &str) -> bool {
        a != b
    }

    fn boolean(a: bool, b: bool) -> bool {
        a != b
    }
}

pub(crate) struct LessThanOp;

impl ComparisonOp for LessThanOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt,
    {
        a < b
    }

    fn decimal(a: Decimal, b: Decimal) -> bool {
        a < b
    }

    fn float<F>(a: F, b: F) -> bool
    where
        F: Float,
    {
        a < b
    }

    fn string(a: &str, b: &str) -> bool {
        a < b
    }

    #[allow(clippy::bool_comparison)]
    fn boolean(a: bool, b: bool) -> bool {
        a < b
    }
}

pub(crate) struct LessThanOrEqualOp;

impl ComparisonOp for LessThanOrEqualOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt,
    {
        a <= b
    }

    fn decimal(a: Decimal, b: Decimal) -> bool {
        a <= b
    }

    fn float<F>(a: F, b: F) -> bool
    where
        F: Float,
    {
        a <= b
    }

    fn string(a: &str, b: &str) -> bool {
        a <= b
    }

    #[allow(clippy::bool_comparison)]
    fn boolean(a: bool, b: bool) -> bool {
        a <= b
    }
}

pub(crate) struct GreaterThanOp;

impl ComparisonOp for GreaterThanOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt,
    {
        a > b
    }

    fn decimal(a: Decimal, b: Decimal) -> bool {
        a > b
    }

    fn float<F>(a: F, b: F) -> bool
    where
        F: Float,
    {
        a > b
    }

    fn string(a: &str, b: &str) -> bool {
        a > b
    }

    #[allow(clippy::bool_comparison)]
    fn boolean(a: bool, b: bool) -> bool {
        a > b
    }
}

pub(crate) struct GreaterThanOrEqualOp;

impl ComparisonOp for GreaterThanOrEqualOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt,
    {
        a >= b
    }

    fn decimal(a: Decimal, b: Decimal) -> bool {
        a >= b
    }

    fn float<F>(a: F, b: F) -> bool
    where
        F: Float,
    {
        a >= b
    }

    fn string(a: &str, b: &str) -> bool {
        a >= b
    }

    #[allow(clippy::bool_comparison)]
    fn boolean(a: bool, b: bool) -> bool {
        a >= b
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_compare_bytes() {
        let a: atomic::Atomic = 1i8.into();
        let b: atomic::Atomic = 2i8.into();

        assert!(!comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_cast_untyped() {
        let a: atomic::Atomic = "foo".into();
        let b: atomic::Atomic = atomic::Atomic::Untyped(Rc::new("foo".to_string()));

        assert!(comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_cast_decimal_to_double() {
        let a: atomic::Atomic = dec!(1.5).into();
        let b: atomic::Atomic = 1.5f64.into();

        assert!(comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_byte_and_integer() {
        let a: atomic::Atomic = 1i8.into();
        let b: atomic::Atomic = 1i64.into();

        assert!(comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_integer_and_integer() {
        let a: atomic::Atomic = 1i64.into();
        let b: atomic::Atomic = 1i64.into();

        assert!(comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!comparison_op::<NotEqualOp>(a, b).unwrap());
    }
}
