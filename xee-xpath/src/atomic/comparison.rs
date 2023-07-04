use ibig::IBig;
use num_traits::Float;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use xee_schema_type::BaseNumericType;

use crate::error;

use super::atomic_core as atomic;

pub(crate) fn value_comparison_op<O>(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<bool>
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
        (atomic::Atomic::Integer(a), atomic::Atomic::Integer(b)) => <O as ComparisonOp>::ibig(a, b),
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

    let a_numeric_type = a.schema_type().base_numeric_type();
    let b_numeric_type = b.schema_type().base_numeric_type();

    match (a_numeric_type, b_numeric_type) {
        (None, None) | (_, None) | (None, _) => {
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
        }

        (Some(a_numeric_type), Some(b_numeric_type)) => {
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

fn cast_untyped(value: atomic::Atomic) -> atomic::Atomic {
    if let atomic::Atomic::Untyped(s) = value {
        atomic::Atomic::String(s)
    } else {
        value
    }
}

pub(crate) trait ComparisonOp {
    fn ibig(a: IBig, b: IBig) -> bool;
    fn decimal(a: Decimal, b: Decimal) -> bool;
    fn float<F>(a: F, b: F) -> bool
    where
        F: Float;
    fn string(a: &str, b: &str) -> bool;
    fn boolean(a: bool, b: bool) -> bool;
}

pub(crate) struct EqualOp;

impl ComparisonOp for EqualOp {
    fn ibig(a: IBig, b: IBig) -> bool {
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
    fn ibig(a: IBig, b: IBig) -> bool {
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
    fn ibig(a: IBig, b: IBig) -> bool {
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
    fn ibig(a: IBig, b: IBig) -> bool {
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
    fn ibig(a: IBig, b: IBig) -> bool {
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
    fn ibig(a: IBig, b: IBig) -> bool {
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
