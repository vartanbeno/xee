use num_traits::{Float, PrimInt};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

pub(crate) fn comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<bool>
where
    O: ComparisonOp,
{
    let (a, b) = cast_untyped(a, b)?;

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

fn cast_untyped(
    a: atomic::Atomic,
    b: atomic::Atomic,
) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
    let r = match (&a, &b) {
        // If both atomic values are instances of xs:untypedAtomic, then the
        // values are cast to the type xs:string.
        (atomic::Atomic::Untyped(a), atomic::Atomic::Untyped(b)) => (
            atomic::Atomic::String(a.clone()),
            atomic::Atomic::String(b.clone()),
        ),
        // If exactly one of the atomic values is an instance of
        // xs:untypedAtomic, it is cast to a type depending on the other
        // value's dynamic type T according to the following rules, in which V
        // denotes the value to be cast:
        (atomic::Atomic::Untyped(a), _) => {
            let a = b.general_comparison_cast(a)?;
            (a, b.clone())
        }
        (_, atomic::Atomic::Untyped(b)) => {
            let b = a.general_comparison_cast(b)?;
            (a.clone(), b)
        }
        _ => (a, b),
    };
    Ok(r)
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
    use super::*;

    #[test]
    fn test_compare_bytes() {
        let a: atomic::Atomic = 1i8.into();
        let b: atomic::Atomic = 2i8.into();

        assert!(!comparison_op::<EqualOp>(a.clone(), b.clone()).unwrap());
        assert!(comparison_op::<NotEqualOp>(a, b).unwrap());
    }
}
