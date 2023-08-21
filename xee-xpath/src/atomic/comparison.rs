use std::rc::Rc;

use ibig::IBig;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

use super::cast_datetime::YearMonthDuration;
use super::cast_numeric::cast_numeric_binary;

// simulate trait alias
pub(crate) trait ComparisonOps:
    ComparisonOp<Rc<String>>
    + ComparisonOp<bool>
    + ComparisonOp<Rc<Decimal>>
    + ComparisonOp<Rc<IBig>>
    + ComparisonOp<OrderedFloat<f32>>
    + ComparisonOp<OrderedFloat<f64>>
    + ComparisonOp<YearMonthDuration>
    + ComparisonOp<Rc<chrono::Duration>>
{
}

impl<
        T: ComparisonOp<Rc<String>>
            + ComparisonOp<bool>
            + ComparisonOp<Rc<Decimal>>
            + ComparisonOp<Rc<IBig>>
            + ComparisonOp<OrderedFloat<f32>>
            + ComparisonOp<OrderedFloat<f64>>
            + ComparisonOp<YearMonthDuration>
            + ComparisonOp<Rc<chrono::Duration>>,
    > ComparisonOps for T
{
}

pub(crate) fn value_comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<bool>
where
    O: ComparisonOps,
{
    let (a, b) = cast(a, b)?;

    // cast guarantees both atomic types are the same concrete atomic
    Ok(match (a, b) {
        (atomic::Atomic::String(_, a), atomic::Atomic::String(_, b)) => {
            <O as ComparisonOp<Rc<String>>>::compare(a, b)
        }
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => {
            <O as ComparisonOp<bool>>::compare(a, b)
        }
        (atomic::Atomic::Decimal(a), atomic::Atomic::Decimal(b)) => {
            <O as ComparisonOp<Rc<Decimal>>>::compare(a, b)
        }
        (atomic::Atomic::Integer(_, a), atomic::Atomic::Integer(_, b)) => {
            <O as ComparisonOp<Rc<IBig>>>::compare(a, b)
        }
        (atomic::Atomic::Float(a), atomic::Atomic::Float(b)) => {
            <O as ComparisonOp<OrderedFloat<f32>>>::compare(a, b)
        }
        (atomic::Atomic::Double(a), atomic::Atomic::Double(b)) => {
            <O as ComparisonOp<OrderedFloat<f64>>>::compare(a, b)
        }
        (atomic::Atomic::YearMonthDuration(a), atomic::Atomic::YearMonthDuration(b)) => {
            <O as ComparisonOp<YearMonthDuration>>::compare(a, b)
        }
        (atomic::Atomic::DayTimeDuration(a), atomic::Atomic::DayTimeDuration(b)) => {
            <O as ComparisonOp<Rc<chrono::Duration>>>::compare(a, b)
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

    cast_numeric_binary(a, b, |a, b| {
        if a.has_same_schema_type(&b) {
            // if both are non-numeric (already handled) and the same type,
            // they are comparable
            Ok((a, b))
        } else {
            // We're not handling derived non-atomic data types,
            // which is okay as atomization has taking place already
            // 5d otherwise, type error
            Err(error::Error::Type)
        }
    })
}

fn cast_untyped(value: atomic::Atomic) -> atomic::Atomic {
    if let atomic::Atomic::Untyped(s) = value {
        atomic::Atomic::String(atomic::StringType::String, s)
    } else {
        value
    }
}

pub(crate) trait ComparisonOp<V>
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool;
}

pub(crate) struct EqualOp;

impl<V> ComparisonOp<V> for EqualOp
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool {
        a == b
    }
}

pub(crate) struct NotEqualOp;

impl<V> ComparisonOp<V> for NotEqualOp
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool {
        a != b
    }
}

pub(crate) struct LessThanOp;

impl<V> ComparisonOp<V> for LessThanOp
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool {
        a < b
    }
}

pub(crate) struct LessThanOrEqualOp;

impl<V> ComparisonOp<V> for LessThanOrEqualOp
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool {
        a <= b
    }
}
pub(crate) struct GreaterThanOp;

impl<V> ComparisonOp<V> for GreaterThanOp
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool {
        a > b
    }
}

pub(crate) struct GreaterThanOrEqualOp;

impl<V> ComparisonOp<V> for GreaterThanOrEqualOp
where
    V: Eq + Ord,
{
    fn compare(a: V, b: V) -> bool {
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
