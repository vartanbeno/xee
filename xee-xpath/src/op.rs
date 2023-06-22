use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::stack;

pub(crate) fn numeric_add(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<stack::Atomic> {
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<stack::Atomic> {
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<stack::Atomic> {
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<stack::Atomic> {
    match (atomic_a, atomic_b) {
        // As a special case, if the types of both $arg1 and $arg2 are
        // xs:integer, then the return type is xs:decimal.
        (stack::Atomic::Integer(_), stack::Atomic::Integer(_)) => numeric_divide(
            &stack::Atomic::Decimal(atomic_a.convert_to_decimal().unwrap()),
            &stack::Atomic::Decimal(atomic_b.convert_to_decimal().unwrap()),
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<stack::Atomic> {
    // A dynamic error is raised [err:FOAR0001] if the divisor is (positive or negative) zero.
    if atomic_b.is_zero() {
        return Err(stack::Error::DivisionByZero);
    }
    // A dynamic error is raised [err:FOAR0002] if either operand is NaN or if $arg1 is INF or -INF.
    if atomic_a.is_nan() || atomic_b.is_nan() || atomic_a.is_infinite() {
        return Err(stack::Error::Overflow);
    }
    match numeric_divide(atomic_a, atomic_b)? {
        stack::Atomic::Integer(i) => Ok(stack::Atomic::Integer(i)),
        stack::Atomic::Decimal(d) => Ok(stack::Atomic::Integer(d.trunc().to_i64().unwrap())),
        stack::Atomic::Float(f) => Ok(stack::Atomic::Integer(f.trunc() as i64)),
        stack::Atomic::Double(d) => Ok(stack::Atomic::Integer(d.trunc() as i64)),
        _ => unreachable!(),
    }
}

pub(crate) fn numeric_mod(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<stack::Atomic> {
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

pub(crate) fn numeric_unary_plus(atomic: &stack::Atomic) -> stack::Result<stack::Atomic> {
    match atomic {
        stack::Atomic::Integer(_) => Ok(atomic.clone()),
        stack::Atomic::Decimal(_) => Ok(atomic.clone()),
        stack::Atomic::Float(_) => Ok(atomic.clone()),
        stack::Atomic::Double(_) => Ok(atomic.clone()),
        // XXX function conversion rules?
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn numeric_unary_minus(atomic: &stack::Atomic) -> stack::Result<stack::Atomic> {
    match atomic {
        stack::Atomic::Integer(i) => Ok(stack::Atomic::Integer(-i)),
        stack::Atomic::Decimal(d) => Ok(stack::Atomic::Decimal(-d)),
        stack::Atomic::Float(f) => Ok(stack::Atomic::Float(-f)),
        stack::Atomic::Double(d) => Ok(stack::Atomic::Double(-d)),
        // XXX function conversion rules?
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn numeric_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => Ok(a == b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_not_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => Ok(a != b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_less_than(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => Ok(a < b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_less_than_or_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => Ok(a <= b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_greater_than(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => Ok(a > b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn string_greater_than_or_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::String(a), stack::Atomic::String(b)) => Ok(a >= b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => Ok(a == b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_not_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => Ok(a != b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_less_than(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => Ok(a < b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_less_than_or_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => Ok(a <= b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_greater_than(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => Ok(a > b),
        _ => Err(stack::Error::Type),
    }
}

pub(crate) fn boolean_greater_than_or_equal(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
) -> stack::Result<bool> {
    match (atomic_a, atomic_b) {
        (stack::Atomic::Boolean(a), stack::Atomic::Boolean(b)) => Ok(a >= b),
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
    ops: ArithmeticOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>,
) -> stack::Result<stack::Atomic>
where
    IntegerOp: FnOnce(i64, i64) -> stack::Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> stack::Result<Decimal>,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> OrderedFloat<f32>,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> OrderedFloat<f64>,
{
    numeric_general_op(atomic_a, atomic_b, |atomic_a, atomic_b| {
        match (atomic_a, atomic_b) {
            (stack::Atomic::Integer(a), stack::Atomic::Integer(b)) => {
                Ok(stack::Atomic::Integer((ops.integer_op)(*a, *b)?))
            }
            (stack::Atomic::Decimal(a), stack::Atomic::Decimal(b)) => {
                Ok(stack::Atomic::Decimal((ops.decimal_op)(*a, *b)?))
            }
            (stack::Atomic::Float(a), stack::Atomic::Float(b)) => {
                Ok(stack::Atomic::Float((ops.float_op)(*a, *b)))
            }
            (stack::Atomic::Double(a), stack::Atomic::Double(b)) => {
                Ok(stack::Atomic::Double((ops.double_op)(*a, *b)))
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
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
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
            (stack::Atomic::Integer(a), stack::Atomic::Integer(b)) => Ok((ops.integer_op)(*a, *b)),
            (stack::Atomic::Decimal(a), stack::Atomic::Decimal(b)) => Ok((ops.decimal_op)(*a, *b)),
            (stack::Atomic::Float(a), stack::Atomic::Float(b)) => Ok((ops.float_op)(*a, *b)),
            (stack::Atomic::Double(a), stack::Atomic::Double(b)) => Ok((ops.double_op)(*a, *b)),
            _ => unreachable!("Illegal combination"),
        }
    })
}

fn numeric_general_op<F, V>(
    atomic_a: &stack::Atomic,
    atomic_b: &stack::Atomic,
    op: F,
) -> stack::Result<V>
where
    F: FnOnce(&stack::Atomic, &stack::Atomic) -> stack::Result<V>,
{
    // S - type substition due to type hierarchy
    //     https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
    // P - type promotion:
    //    float -> double
    //    decimal -> float
    //    decimal -> double
    match (atomic_a, atomic_b) {
        // -> integer
        (stack::Atomic::Integer(_), stack::Atomic::Integer(_)) => op(atomic_a, atomic_b),
        // -> decimal
        (stack::Atomic::Decimal(_), stack::Atomic::Decimal(_)) => op(atomic_a, atomic_b),
        (stack::Atomic::Integer(_), stack::Atomic::Decimal(_)) => {
            // integer S decimal
            op(
                &stack::Atomic::Decimal(atomic_a.convert_to_decimal()?),
                atomic_b,
            )
        }
        (stack::Atomic::Decimal(_), stack::Atomic::Integer(_)) => {
            // integer S decimal
            op(
                atomic_a,
                &stack::Atomic::Decimal(atomic_b.convert_to_decimal()?),
            )
        }
        // -> float
        (stack::Atomic::Float(_), stack::Atomic::Float(_)) => op(atomic_a, atomic_b),
        (stack::Atomic::Decimal(_), stack::Atomic::Float(_)) => {
            // decimal P float
            op(
                &stack::Atomic::Float(atomic_a.convert_to_float()?),
                atomic_b,
            )
        }
        (stack::Atomic::Integer(_), stack::Atomic::Float(_)) => {
            // integer S decimal P float
            op(
                &stack::Atomic::Float(atomic_a.convert_to_float()?),
                atomic_b,
            )
        }
        (stack::Atomic::Float(_), stack::Atomic::Decimal(_)) => {
            // decimal P float
            op(
                atomic_a,
                &stack::Atomic::Float(atomic_b.convert_to_float()?),
            )
        }
        (stack::Atomic::Float(_), stack::Atomic::Integer(_)) => {
            // integer S decimal P float
            op(
                atomic_a,
                &stack::Atomic::Float(atomic_b.convert_to_float()?),
            )
        }
        // -> double
        (stack::Atomic::Double(_), stack::Atomic::Double(_)) => op(atomic_a, atomic_b),
        (stack::Atomic::Decimal(_), stack::Atomic::Double(_)) => {
            // decimal P double
            op(
                &stack::Atomic::Double(atomic_a.convert_to_double()?),
                atomic_b,
            )
        }
        (stack::Atomic::Integer(_), stack::Atomic::Double(_)) => {
            // integer S decimal P double
            op(
                &stack::Atomic::Double(atomic_a.convert_to_double()?),
                atomic_b,
            )
        }
        (stack::Atomic::Double(_), stack::Atomic::Decimal(_)) => {
            // decimal P double
            op(
                atomic_a,
                &stack::Atomic::Double(atomic_b.convert_to_double()?),
            )
        }
        (stack::Atomic::Double(_), stack::Atomic::Integer(_)) => {
            // integer S decimal P double
            op(
                atomic_a,
                &stack::Atomic::Double(atomic_b.convert_to_double()?),
            )
        }
        (stack::Atomic::Float(_), stack::Atomic::Double(_)) => {
            // float P double
            op(
                &stack::Atomic::Double(atomic_a.convert_to_double()?),
                atomic_b,
            )
        }
        (stack::Atomic::Double(_), stack::Atomic::Float(_)) => {
            // float P double
            op(
                atomic_a,
                &stack::Atomic::Double(atomic_b.convert_to_double()?),
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
            numeric_add(&stack::Atomic::Integer(1), &stack::Atomic::Integer(2)).unwrap(),
            stack::Atomic::Integer(3)
        );
    }

    #[test]
    fn test_add_integers_overflow() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Integer(i64::MAX),
                &stack::Atomic::Integer(2)
            ),
            Err(stack::Error::Overflow)
        );
    }

    #[test]
    fn test_add_decimals() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Decimal(dec!(1.5)),
                &stack::Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            stack::Atomic::Decimal(Decimal::new(42, 1))
        );
    }

    #[test]
    fn test_add_decimals_overflow() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Decimal(Decimal::MAX),
                &stack::Atomic::Decimal(dec!(2.7))
            ),
            Err(stack::Error::Overflow)
        );
    }

    #[test]
    fn test_add_floats() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Float(OrderedFloat(1.5)),
                &stack::Atomic::Float(OrderedFloat(2.7))
            )
            .unwrap(),
            stack::Atomic::Float(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_add_doubles() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Double(OrderedFloat(1.5)),
                &stack::Atomic::Double(OrderedFloat(2.7))
            )
            .unwrap(),
            stack::Atomic::Double(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_add_integer_decimal() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Integer(1),
                &stack::Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            stack::Atomic::Decimal(Decimal::new(37, 1))
        );
    }

    #[test]
    fn test_add_double_decimal() {
        assert_eq!(
            numeric_add(
                &stack::Atomic::Double(OrderedFloat(1.5)),
                &stack::Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            stack::Atomic::Double(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_numeric_divide_both_integer_returns_decimal() {
        assert_eq!(
            numeric_divide(&stack::Atomic::Integer(1), &stack::Atomic::Integer(2)).unwrap(),
            stack::Atomic::Decimal(dec!(0.5))
        );
    }

    #[test]
    fn test_numeric_integer_divide_10_by_3() {
        assert_eq!(
            numeric_integer_divide(&stack::Atomic::Integer(10), &stack::Atomic::Integer(3))
                .unwrap(),
            stack::Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_by_minus_2() {
        assert_eq!(
            numeric_integer_divide(&stack::Atomic::Integer(3), &stack::Atomic::Integer(-2))
                .unwrap(),
            stack::Atomic::Integer(-1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_2() {
        assert_eq!(
            numeric_integer_divide(&stack::Atomic::Integer(-3), &stack::Atomic::Integer(2))
                .unwrap(),
            stack::Atomic::Integer(-1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_minus_2() {
        assert_eq!(
            numeric_integer_divide(&stack::Atomic::Integer(-3), &stack::Atomic::Integer(-2))
                .unwrap(),
            stack::Atomic::Integer(1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_9_point_0_by_3() {
        assert_eq!(
            numeric_integer_divide(
                &stack::Atomic::Double(OrderedFloat(9.0)),
                &stack::Atomic::Integer(3)
            )
            .unwrap(),
            stack::Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_4() {
        assert_eq!(
            numeric_integer_divide(
                &stack::Atomic::Double(OrderedFloat(3.0)),
                &stack::Atomic::Integer(4)
            )
            .unwrap(),
            stack::Atomic::Integer(0)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_by_0() {
        assert_eq!(
            numeric_integer_divide(&stack::Atomic::Integer(3), &stack::Atomic::Integer(0)),
            Err(stack::Error::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_0() {
        assert_eq!(
            numeric_integer_divide(
                &stack::Atomic::Double(OrderedFloat(3.0)),
                &stack::Atomic::Integer(0)
            ),
            Err(stack::Error::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_inf() {
        assert_eq!(
            numeric_integer_divide(
                &stack::Atomic::Double(OrderedFloat(3.0)),
                &stack::Atomic::Double(OrderedFloat(f64::INFINITY))
            )
            .unwrap(),
            stack::Atomic::Integer(0)
        );
    }

    #[test]
    fn test_numeric_mod_nan_nan() {
        assert!(numeric_mod(
            &stack::Atomic::Double(OrderedFloat(f64::NAN)),
            &stack::Atomic::Double(OrderedFloat(f64::NAN))
        )
        .unwrap()
        .is_nan());
    }
}
