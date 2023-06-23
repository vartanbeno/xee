use num_traits::{Float, PrimInt};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

fn comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<atomic::Atomic>
where
    O: ComparisonOp,
{
    Ok(match (a, b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => {
            <O as ComparisonOp>::string_atomic(&a, &b)
        }
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => {
            <O as ComparisonOp>::boolean_atomic(a, b)
        }
        (atomic::Atomic::Decimal(a), atomic::Atomic::Decimal(b)) => {
            <O as ComparisonOp>::decimal_atomic(a, b)
        }
        (atomic::Atomic::Integer(a), atomic::Atomic::Integer(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::Int(a), atomic::Atomic::Int(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::Short(a), atomic::Atomic::Short(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::Byte(a), atomic::Atomic::Byte(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::UnsignedLong(a), atomic::Atomic::UnsignedLong(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::UnsignedInt(a), atomic::Atomic::UnsignedInt(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::UnsignedShort(a), atomic::Atomic::UnsignedShort(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::UnsignedByte(a), atomic::Atomic::UnsignedByte(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (atomic::Atomic::Float(OrderedFloat(a)), atomic::Atomic::Float(OrderedFloat(b))) => {
            <O as ComparisonOp>::float_atomic(a, b)
        }
        (atomic::Atomic::Double(OrderedFloat(a)), atomic::Atomic::Double(OrderedFloat(b))) => {
            <O as ComparisonOp>::float_atomic(a, b)
        }
        _ => unreachable!("Both the atomics are not the same type or types aren't handled"),
    })
}

trait ComparisonOp {
    fn integer<I>(a: I, b: I) -> bool
    where
        I: PrimInt;
    fn decimal(a: Decimal, b: Decimal) -> bool;
    fn float<F>(a: F, b: F) -> bool
    where
        F: Float;
    fn string(a: &str, b: &str) -> bool;
    fn boolean(a: bool, b: bool) -> bool;

    fn integer_atomic<I>(a: I, b: I) -> atomic::Atomic
    where
        I: PrimInt + Into<atomic::Atomic> + Into<Decimal>,
    {
        let v = <Self as ComparisonOp>::integer(a, b);
        v.into()
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> atomic::Atomic {
        let v = <Self as ComparisonOp>::decimal(a, b);
        v.into()
    }

    fn float_atomic<F>(a: F, b: F) -> atomic::Atomic
    where
        F: Float + Into<atomic::Atomic>,
    {
        let v = <Self as ComparisonOp>::float(a, b);
        v.into()
    }

    fn string_atomic(a: &str, b: &str) -> atomic::Atomic {
        let v = <Self as ComparisonOp>::string(a, b);
        v.into()
    }

    fn boolean_atomic(a: bool, b: bool) -> atomic::Atomic {
        let v = <Self as ComparisonOp>::boolean(a, b);
        v.into()
    }
}

struct EqualOp;

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

struct NotEqualOp;

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

struct LessThanOp;

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

struct LessThanOrEqualOp;

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

struct GreaterThanOp;

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

struct GreaterThanOrEqualOp;

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
