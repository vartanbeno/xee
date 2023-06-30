use num_traits::{Float, PrimInt};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use xee_schema_type::Xs;

use crate::error;

use super::atomic_core as atomic;

pub(crate) fn value_comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<bool>
where
    O: ComparisonOp,
{
    let (a, b) = cast(a, b)?;
    comparison_op::<O>(a, b)
}

pub(crate) fn comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<bool>
where
    O: ComparisonOp,
{
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

enum BaseType {
    Decimal,
    Float,
    Double,
    Other,
}

fn base_type(a: &atomic::Atomic) -> BaseType {
    if a.has_base_schema_type(Xs::Decimal) {
        BaseType::Decimal
    } else if a.has_base_schema_type(Xs::Float) {
        BaseType::Float
    } else if a.has_base_schema_type(Xs::Double) {
        BaseType::Double
    } else {
        BaseType::Other
    }
}

fn cast(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
    // 3.7.1 Value Comparisons
    // We start in step 4, as the previous steps have been handled
    // by the caller.

    // 4: If an atomized operand of of type xs:untypedAtomic, it is cast
    // to xs:string
    let a = cast_untyped(a);
    let b = cast_untyped(b);

    let a_base_type = base_type(&a);
    let b_base_type = base_type(&b);

    match (a_base_type, b_base_type) {
        // 5a: TODO: xs:string and xs:anyURI
        // 5b: xs:decimal & xs:float -> cast decimal to float
        (BaseType::Decimal, BaseType::Float) => Ok((a.cast_to_float()?, b)),
        (BaseType::Float, BaseType::Decimal) => Ok((a, b.cast_to_float()?)),
        // 5c: xs:decimal & xs:double -> cast decimal to double
        (BaseType::Decimal, BaseType::Double) => Ok((a.cast_to_double()?, b)),
        (BaseType::Double, BaseType::Decimal) => Ok((a, b.cast_to_double()?)),
        // 5c: xs:float & xs:double -> cast float to double
        (BaseType::Float, BaseType::Double) => Ok((a.cast_to_double()?, b)),
        (BaseType::Double, BaseType::Float) => Ok((a, b.cast_to_double()?)),

        // decimal types need to be made the same
        (BaseType::Decimal, BaseType::Decimal) => a.cast_to_same_schema_type(&b),
        // float and double types are already the same
        (BaseType::Float, BaseType::Float) => Ok((a, b)),
        (BaseType::Double, BaseType::Double) => Ok((a, b)),
        // any other type can be compared if the types are the same
        (BaseType::Other, _) | (_, BaseType::Other) => {
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

        assert!(!value_comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(value_comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_cast_untyped() {
        let a: atomic::Atomic = "foo".into();
        let b: atomic::Atomic = atomic::Atomic::Untyped(Rc::new("foo".to_string()));

        assert!(value_comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!value_comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_cast_decimal_to_double() {
        let a: atomic::Atomic = dec!(1.5).into();
        let b: atomic::Atomic = 1.5f64.into();

        assert!(value_comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!value_comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_byte_and_integer() {
        let a: atomic::Atomic = 1i8.into();
        let b: atomic::Atomic = 1i64.into();

        assert!(value_comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!value_comparison_op::<NotEqualOp>(a, b).unwrap());
    }

    #[test]
    fn test_compare_integer_and_integer() {
        let a: atomic::Atomic = 1i64.into();
        let b: atomic::Atomic = 1i64.into();

        assert!(value_comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(!value_comparison_op::<NotEqualOp>(a, b).unwrap());
    }
}
