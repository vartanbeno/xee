use num_traits::{Float, PrimInt};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::stack;

fn comparison_op<O>(a: stack::Atomic, b: stack::Atomic) -> stack::Result<stack::Atomic>
where
    O: ComparisonOp,
{
    Ok(match (a, b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => {
            <O as ComparisonOp>::string_atomic(&a, &b)
        }
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => {
            <O as ComparisonOp>::boolean_atomic(a, b)
        }
        (stack::Atomic::Decimal(a), stack::Atomic::Decimal(b)) => {
            <O as ComparisonOp>::decimal_atomic(a, b)
        }
        (stack::Atomic::Integer(a), stack::Atomic::Integer(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::Int(a), stack::Atomic::Int(b)) => <O as ComparisonOp>::integer_atomic(a, b),
        (stack::Atomic::Short(a), stack::Atomic::Short(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::Byte(a), stack::Atomic::Byte(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::UnsignedLong(a), stack::Atomic::UnsignedLong(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::UnsignedInt(a), stack::Atomic::UnsignedInt(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::UnsignedShort(a), stack::Atomic::UnsignedShort(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::UnsignedByte(a), stack::Atomic::UnsignedByte(b)) => {
            <O as ComparisonOp>::integer_atomic(a, b)
        }
        (stack::Atomic::Float(OrderedFloat(a)), stack::Atomic::Float(OrderedFloat(b))) => {
            <O as ComparisonOp>::float_atomic(a, b)
        }
        (stack::Atomic::Double(OrderedFloat(a)), stack::Atomic::Double(OrderedFloat(b))) => {
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

    fn integer_atomic<I>(a: I, b: I) -> stack::Atomic
    where
        I: PrimInt + Into<stack::Atomic> + Into<Decimal>,
    {
        let v = <Self as ComparisonOp>::integer(a, b);
        v.into()
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> stack::Atomic {
        let v = <Self as ComparisonOp>::decimal(a, b);
        v.into()
    }

    fn float_atomic<F>(a: F, b: F) -> stack::Atomic
    where
        F: Float + Into<stack::Atomic>,
    {
        let v = <Self as ComparisonOp>::float(a, b);
        v.into()
    }

    fn string_atomic(a: &str, b: &str) -> stack::Atomic {
        let v = <Self as ComparisonOp>::string(a, b);
        v.into()
    }

    fn boolean_atomic(a: bool, b: bool) -> stack::Atomic {
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
