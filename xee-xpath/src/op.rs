use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::stack;

pub(crate) fn numeric_add(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<atomic::Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| a.checked_add(b).ok_or(stack::Error::Overflow),
            decimal_op: |a, b| a.checked_add(b).ok_or(stack::Error::Overflow),
            float_op: |a, b| a + b,
            double_op: |a, b| a + b,
        },
    )
}

pub(crate) fn numeric_substract(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<atomic::Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| a.checked_sub(b).ok_or(stack::Error::Overflow),
            decimal_op: |a, b| a.checked_sub(b).ok_or(stack::Error::Overflow),
            float_op: |a, b| a - b,
            double_op: |a, b| a - b,
        },
    )
}

pub(crate) fn numeric_multiply(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<atomic::Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| a.checked_mul(b).ok_or(stack::Error::Overflow),
            decimal_op: |a, b| a.checked_mul(b).ok_or(stack::Error::Overflow),
            float_op: |a, b| a * b,
            double_op: |a, b| a * b,
        },
    )
}

pub(crate) fn numeric_divide(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<atomic::Atomic> {
    match (atomic_a, atomic_b) {
        // As a special case, if the types of both $arg1 and $arg2 are
        // xs:integer, then the return type is xs:decimal.
        (atomic::Atomic::Integer(_), atomic::Atomic::Integer(_)) => numeric_divide(
            &atomic::Atomic::Decimal(atomic_a.convert_to_decimal().unwrap()),
            &atomic::Atomic::Decimal(atomic_b.convert_to_decimal().unwrap()),
        ),
        _ => {
            arithmetic_op(
                atomic_a,
                atomic_b,
                ArithmeticOps {
                    integer_op: |a, b| {
                        if b == 0 {
                            Err(stack::Error::DivisionByZero)
                        } else {
                            Ok(a / b)
                        }
                    },
                    decimal_op: |a, b| {
                        if b.is_zero() {
                            Err(stack::Error::DivisionByZero)
                        } else {
                            Ok(a / b)
                        }
                    },
                    // For xs:float and xs:double operands, floating point division is
                    // performed as specified in [IEEE 754-2008].
                    // Returns INF, INF or NaN
                    float_op: |a, b| a / b,
                    double_op: |a, b| a / b,
                },
            )
        }
    }
}

pub(crate) fn numeric_integer_divide(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<atomic::Atomic> {
    // A dynamic error is raised [err:FOAR0001] if the divisor is (positive or negative) zero.
    if atomic_b.is_zero() {
        return Err(stack::Error::DivisionByZero);
    }
    // A dynamic error is raised [err:FOAR0002] if either operand is NaN or if $arg1 is INF or -INF.
    if atomic_a.is_nan() || atomic_b.is_nan() || atomic_a.is_infinite() {
        return Err(stack::Error::Overflow);
    }
    match numeric_divide(atomic_a, atomic_b)? {
        atomic::Atomic::Integer(i) => Ok(atomic::Atomic::Integer(i)),
        atomic::Atomic::Decimal(d) => Ok(atomic::Atomic::Integer(d.trunc().to_i64().unwrap())),
        atomic::Atomic::Float(f) => Ok(atomic::Atomic::Integer(f.trunc() as i64)),
        atomic::Atomic::Double(d) => Ok(atomic::Atomic::Integer(d.trunc() as i64)),
        _ => unreachable!(),
    }
}

pub(crate) fn numeric_mod(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<atomic::Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| {
                if b == 0 {
                    Err(stack::Error::DivisionByZero)
                } else {
                    Ok(a % b)
                }
            },
            decimal_op: |a, b| {
                if b.is_zero() {
                    Err(stack::Error::DivisionByZero)
                } else {
                    Ok(a % b)
                }
            },
            float_op: |a, b| a % b,
            double_op: |a, b| a % b,
        },
    )
}

pub(crate) fn numeric_unary_plus(atomic: &atomic::Atomic) -> stack::Result<atomic::Atomic> {
    match atomic {
        atomic::Atomic::Integer(_) => Ok(atomic.clone()),
        atomic::Atomic::Decimal(_) => Ok(atomic.clone()),
        atomic::Atomic::Float(_) => Ok(atomic.clone()),
        atomic::Atomic::Double(_) => Ok(atomic.clone()),
        // XXX function conversion rules?
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn numeric_unary_minus(atomic: &atomic::Atomic) -> stack::Result<atomic::Atomic> {
    match atomic {
        atomic::Atomic::Integer(i) => Ok(atomic::Atomic::Integer(-i)),
        atomic::Atomic::Decimal(d) => Ok(atomic::Atomic::Decimal(-d)),
        atomic::Atomic::Float(f) => Ok(atomic::Atomic::Float(-f)),
        atomic::Atomic::Double(d) => Ok(atomic::Atomic::Double(-d)),
        // XXX function conversion rules?
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn numeric_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
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
) -> stack::Result<bool> {
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
) -> stack::Result<bool> {
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
) -> stack::Result<bool> {
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
) -> stack::Result<bool> {
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
) -> stack::Result<bool> {
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
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a == b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_not_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a != b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_less_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a < b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_less_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a <= b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_greater_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a > b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_greater_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::String(a), atomic::Atomic::String(b)) => Ok(a >= b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a == b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_not_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a != b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_less_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a < b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_less_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a <= b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_greater_than(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a > b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_greater_than_or_equal(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (atomic::Atomic::Boolean(a), atomic::Atomic::Boolean(b)) => Ok(a >= b),
        _ => Err(stack::Error::Type),
    }
}

struct ArithmeticOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>
where
    IntegerOp: FnOnce(i64, i64) -> stack::Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> stack::Result<Decimal>,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> OrderedFloat<f32>,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> OrderedFloat<f64>,
{
    integer_op: IntegerOp,
    decimal_op: DecimalOp,
    float_op: FloatOp,
    double_op: DoubleOp,
}

fn arithmetic_op<IntegerOp, DecimalOp, FloatOp, DoubleOp>(
    atomic_a: &atomic::Atomic,
    atomic_b: &atomic::Atomic,
    ops: ArithmeticOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>,
) -> stack::Result<atomic::Atomic>
where
    IntegerOp: FnOnce(i64, i64) -> stack::Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> stack::Result<Decimal>,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> OrderedFloat<f32>,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> OrderedFloat<f64>,
{
    numeric_general_op(atomic_a, atomic_b, |atomic_a, atomic_b| {
        match (atomic_a, atomic_b) {
            (atomic::Atomic::Integer(a), atomic::Atomic::Integer(b)) => {
                Ok(atomic::Atomic::Integer((ops.integer_op)(*a, *b)?))
            }
            (atomic::Atomic::Decimal(a), atomic::Atomic::Decimal(b)) => {
                Ok(atomic::Atomic::Decimal((ops.decimal_op)(*a, *b)?))
            }
            (atomic::Atomic::Float(a), atomic::Atomic::Float(b)) => {
                Ok(atomic::Atomic::Float((ops.float_op)(*a, *b)))
            }
            (atomic::Atomic::Double(a), atomic::Atomic::Double(b)) => {
                Ok(atomic::Atomic::Double((ops.double_op)(*a, *b)))
            }
            _ => unreachable!("Illegal combination"),
        }
    })
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
) -> stack::Result<bool>
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
) -> stack::Result<V>
where
    F: FnOnce(&atomic::Atomic, &atomic::Atomic) -> stack::Result<V>,
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
        _ => Err(stack::Error::Type),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_add_integers() {
        assert_eq!(
            numeric_add(&atomic::Atomic::Integer(1), &atomic::Atomic::Integer(2)).unwrap(),
            atomic::Atomic::Integer(3)
        );
    }

    #[test]
    fn test_add_integers_overflow() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Integer(i64::MAX),
                &atomic::Atomic::Integer(2)
            ),
            Err(stack::Error::Overflow)
        );
    }

    #[test]
    fn test_add_decimals() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Decimal(dec!(1.5)),
                &atomic::Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            atomic::Atomic::Decimal(Decimal::new(42, 1))
        );
    }

    #[test]
    fn test_add_decimals_overflow() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Decimal(Decimal::MAX),
                &atomic::Atomic::Decimal(dec!(2.7))
            ),
            Err(stack::Error::Overflow)
        );
    }

    #[test]
    fn test_add_floats() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Float(OrderedFloat(1.5)),
                &atomic::Atomic::Float(OrderedFloat(2.7))
            )
            .unwrap(),
            atomic::Atomic::Float(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_add_doubles() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Double(OrderedFloat(1.5)),
                &atomic::Atomic::Double(OrderedFloat(2.7))
            )
            .unwrap(),
            atomic::Atomic::Double(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_add_integer_decimal() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Integer(1),
                &atomic::Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            atomic::Atomic::Decimal(Decimal::new(37, 1))
        );
    }

    #[test]
    fn test_add_double_decimal() {
        assert_eq!(
            numeric_add(
                &atomic::Atomic::Double(OrderedFloat(1.5)),
                &atomic::Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            atomic::Atomic::Double(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_numeric_divide_both_integer_returns_decimal() {
        assert_eq!(
            numeric_divide(&atomic::Atomic::Integer(1), &atomic::Atomic::Integer(2)).unwrap(),
            atomic::Atomic::Decimal(dec!(0.5))
        );
    }

    #[test]
    fn test_numeric_integer_divide_10_by_3() {
        assert_eq!(
            numeric_integer_divide(&atomic::Atomic::Integer(10), &atomic::Atomic::Integer(3))
                .unwrap(),
            atomic::Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_by_minus_2() {
        assert_eq!(
            numeric_integer_divide(&atomic::Atomic::Integer(3), &atomic::Atomic::Integer(-2))
                .unwrap(),
            atomic::Atomic::Integer(-1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_2() {
        assert_eq!(
            numeric_integer_divide(&atomic::Atomic::Integer(-3), &atomic::Atomic::Integer(2))
                .unwrap(),
            atomic::Atomic::Integer(-1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_minus_2() {
        assert_eq!(
            numeric_integer_divide(&atomic::Atomic::Integer(-3), &atomic::Atomic::Integer(-2))
                .unwrap(),
            atomic::Atomic::Integer(1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_9_point_0_by_3() {
        assert_eq!(
            numeric_integer_divide(
                &atomic::Atomic::Double(OrderedFloat(9.0)),
                &atomic::Atomic::Integer(3)
            )
            .unwrap(),
            atomic::Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_4() {
        assert_eq!(
            numeric_integer_divide(
                &atomic::Atomic::Double(OrderedFloat(3.0)),
                &atomic::Atomic::Integer(4)
            )
            .unwrap(),
            atomic::Atomic::Integer(0)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_by_0() {
        assert_eq!(
            numeric_integer_divide(&atomic::Atomic::Integer(3), &atomic::Atomic::Integer(0)),
            Err(stack::Error::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_0() {
        assert_eq!(
            numeric_integer_divide(
                &atomic::Atomic::Double(OrderedFloat(3.0)),
                &atomic::Atomic::Integer(0)
            ),
            Err(stack::Error::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_inf() {
        assert_eq!(
            numeric_integer_divide(
                &atomic::Atomic::Double(OrderedFloat(3.0)),
                &atomic::Atomic::Double(OrderedFloat(f64::INFINITY))
            )
            .unwrap(),
            atomic::Atomic::Integer(0)
        );
    }

    #[test]
    fn test_numeric_mod_nan_nan() {
        assert!(numeric_mod(
            &atomic::Atomic::Double(OrderedFloat(f64::NAN)),
            &atomic::Atomic::Double(OrderedFloat(f64::NAN))
        )
        .unwrap()
        .is_nan());
    }
}
