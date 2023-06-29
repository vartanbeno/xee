use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

pub(crate) fn numeric_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    numeric_comparison_op(
        atomic_a,
        atomic_b,
        ComparisonOps {
            integer_op: |a, b| a == b,
            decimal_op: |a, b| a == b,
            float_op: |a, b| a == b,
            double_op: |a, b| a == b,
        },
    )
}
pub(crate) fn numeric_not_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    numeric_comparison_op(
        atomic_a,
        atomic_b,
        ComparisonOps {
            integer_op: |a, b| a != b,
            decimal_op: |a, b| a != b,
            float_op: |a, b| a != b,
            double_op: |a, b| a != b,
        },
    )
}

pub(crate) fn numeric_less_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    numeric_comparison_op(
        atomic_a,
        atomic_b,
        ComparisonOps {
            integer_op: |a, b| a < b,
            decimal_op: |a, b| a < b,
            float_op: |a, b| a < b,
            double_op: |a, b| a < b,
        },
    )
}

pub(crate) fn numeric_less_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    numeric_comparison_op(
        atomic_a,
        atomic_b,
        ComparisonOps {
            integer_op: |a, b| a <= b,
            decimal_op: |a, b| a <= b,
            float_op: |a, b| a <= b,
            double_op: |a, b| a <= b,
        },
    )
}

pub(crate) fn numeric_greater_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    numeric_comparison_op(
        atomic_a,
        atomic_b,
        ComparisonOps {
            integer_op: |a, b| a > b,
            decimal_op: |a, b| a > b,
            float_op: |a, b| a > b,
            double_op: |a, b| a > b,
        },
    )
}

pub(crate) fn numeric_greater_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    numeric_comparison_op(
        atomic_a,
        atomic_b,
        ComparisonOps {
            integer_op: |a, b| a >= b,
            decimal_op: |a, b| a >= b,
            float_op: |a, b| a >= b,
            double_op: |a, b| a >= b,
        },
    )
}

pub(crate) fn string_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a == b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn string_not_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a != b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn string_less_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a < b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn string_less_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a <= b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn string_greater_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a > b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn string_greater_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a >= b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn boolean_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a == b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn boolean_not_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a != b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn boolean_less_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a < b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn boolean_less_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a <= b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn boolean_greater_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a > b),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn boolean_greater_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> error::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a >= b),
        _ => Err(error::Error::Type),
    }
}
struct ComparisonOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>
where
    IntegerOp: FnOnce(i64, i64) -> bool,
    DecimalOp: FnOnce(Decimal, Decimal) -> bool,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> bool,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> bool,
{
    integer_op: IntegerOp,
    decimal_op: DecimalOp,
    float_op: FloatOp,
    double_op: DoubleOp,
}

fn numeric_comparison_op<IntegerOp, DecimalOp, FloatOp, DoubleOp>(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
    ops: ComparisonOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>,
) -> error::Result<bool>
where
    IntegerOp: FnOnce(i64, i64) -> bool,
    DecimalOp: FnOnce(Decimal, Decimal) -> bool,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> bool,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> bool,
{
    numeric_general_op(atomic_a, atomic_b, |atomic_a, atomic_b| {
        match (atomic_a, atomic_b) {
            (atomic::Atomic::Integer(a), atomic::Atomic::Integer(b)) => {
                Ok((ops.integer_op)(*a, *b))
            }
            (atomic::Atomic::Decimal(a), atomic::Atomic::Decimal(b)) => {
                Ok((ops.decimal_op)(*a, *b))
            }
            (atomic::Atomic::Float(a), atomic::Atomic::Float(b)) => Ok((ops.float_op)(*a, *b)),
            (atomic::Atomic::Double(a), atomic::Atomic::Double(b)) => Ok((ops.double_op)(*a, *b)),
            _ => unreachable!("Illegal combination"),
        }
    })
}

fn numeric_general_op<F, V>(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
    op: F,
) -> error::Result<V>
where
    F: FnOnce(&atomic::Atomic, &atomic::Atomic) -> error::Result<V>,
{
    // S - type substition due to type hierarchy
    //     https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
    // P - type promotion:
    //    float -> double
    //    decimal -> float
    //    decimal -> double
    match (atomic_a, atomic_b) {
        // -> integer
        (atomic::Atomic::Integer(_), atomic::Atomic::Integer(_)) => op(atomic_a, atomic_b),
        // -> decimal
        (atomic::Atomic::Decimal(_), atomic::Atomic::Decimal(_)) => op(atomic_a, atomic_b),
        (atomic::Atomic::Integer(_), atomic::Atomic::Decimal(_)) => {
            // integer S decimal
            op(
                &atomic::Atomic::Decimal(atomic_a.convert_to_decimal()?),
                atomic_b,
            )
        }
        (atomic::Atomic::Decimal(_), atomic::Atomic::Integer(_)) => {
            // integer S decimal
            op(
                atomic_a,
                &atomic::Atomic::Decimal(atomic_b.convert_to_decimal()?),
            )
        }
        // -> float
        (atomic::Atomic::Float(_), atomic::Atomic::Float(_)) => op(atomic_a, atomic_b),
        (atomic::Atomic::Decimal(_), atomic::Atomic::Float(_)) => {
            // decimal P float
            op(
                &atomic::Atomic::Float(atomic_a.convert_to_float()?),
                atomic_b,
            )
        }
        (atomic::Atomic::Integer(_), atomic::Atomic::Float(_)) => {
            // integer S decimal P float
            op(
                &atomic::Atomic::Float(atomic_a.convert_to_float()?),
                atomic_b,
            )
        }
        (atomic::Atomic::Float(_), atomic::Atomic::Decimal(_)) => {
            // decimal P float
            op(
                atomic_a,
                &atomic::Atomic::Float(atomic_b.convert_to_float()?),
            )
        }
        (atomic::Atomic::Float(_), atomic::Atomic::Integer(_)) => {
            // integer S decimal P float
            op(
                atomic_a,
                &atomic::Atomic::Float(atomic_b.convert_to_float()?),
            )
        }
        // -> double
        (atomic::Atomic::Double(_), atomic::Atomic::Double(_)) => op(atomic_a, atomic_b),
        (atomic::Atomic::Decimal(_), atomic::Atomic::Double(_)) => {
            // decimal P double
            op(
                &atomic::Atomic::Double(atomic_a.convert_to_double()?),
                atomic_b,
            )
        }
        (atomic::Atomic::Integer(_), atomic::Atomic::Double(_)) => {
            // integer S decimal P double
            op(
                &atomic::Atomic::Double(atomic_a.convert_to_double()?),
                atomic_b,
            )
        }
        (atomic::Atomic::Double(_), atomic::Atomic::Decimal(_)) => {
            // decimal P double
            op(
                atomic_a,
                &atomic::Atomic::Double(atomic_b.convert_to_double()?),
            )
        }
        (atomic::Atomic::Double(_), atomic::Atomic::Integer(_)) => {
            // integer S decimal P double
            op(
                atomic_a,
                &atomic::Atomic::Double(atomic_b.convert_to_double()?),
            )
        }
        (atomic::Atomic::Float(_), atomic::Atomic::Double(_)) => {
            // float P double
            op(
                &atomic::Atomic::Double(atomic_a.convert_to_double()?),
                atomic_b,
            )
        }
        (atomic::Atomic::Double(_), atomic::Atomic::Float(_)) => {
            // float P double
            op(
                atomic_a,
                &atomic::Atomic::Double(atomic_b.convert_to_double()?),
            )
        }
        _ => Err(error::Error::Type),
    }
}
