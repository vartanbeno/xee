use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::data::{Atomic, ValueError};

type Result<T> = std::result::Result<T, ValueError>;

pub(crate) fn numeric_add(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| a.checked_add(b).ok_or(ValueError::Overflow),
            decimal_op: |a, b| a.checked_add(b).ok_or(ValueError::Overflow),
            float_op: |a, b| a + b,
            double_op: |a, b| a + b,
        },
    )
}

pub(crate) fn numeric_substract(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| a.checked_sub(b).ok_or(ValueError::Overflow),
            decimal_op: |a, b| a.checked_sub(b).ok_or(ValueError::Overflow),
            float_op: |a, b| a - b,
            double_op: |a, b| a - b,
        },
    )
}

pub(crate) fn numeric_multiply(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| a.checked_mul(b).ok_or(ValueError::Overflow),
            decimal_op: |a, b| a.checked_mul(b).ok_or(ValueError::Overflow),
            float_op: |a, b| a * b,
            double_op: |a, b| a * b,
        },
    )
}

pub(crate) fn numeric_divide(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    match (atomic_a, atomic_b) {
        // As a special case, if the types of both $arg1 and $arg2 are
        // xs:integer, then the return type is xs:decimal.
        (Atomic::Integer(_), Atomic::Integer(_)) => numeric_divide(
            &Atomic::Decimal(atomic_a.to_decimal().unwrap()),
            &Atomic::Decimal(atomic_b.to_decimal().unwrap()),
        ),
        _ => {
            arithmetic_op(
                atomic_a,
                atomic_b,
                ArithmeticOps {
                    integer_op: |a, b| {
                        if b == 0 {
                            Err(ValueError::DivisionByZero)
                        } else {
                            Ok(a / b)
                        }
                    },
                    decimal_op: |a, b| {
                        if b.is_zero() {
                            Err(ValueError::DivisionByZero)
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

pub(crate) fn numeric_integer_divide(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    // A dynamic error is raised [err:FOAR0001] if the divisor is (positive or negative) zero.
    if atomic_b.is_zero() {
        return Err(ValueError::DivisionByZero);
    }
    // A dynamic error is raised [err:FOAR0002] if either operand is NaN or if $arg1 is INF or -INF.
    if atomic_a.is_nan() || atomic_b.is_nan() || atomic_a.is_infinite() {
        return Err(ValueError::Overflow);
    }
    match numeric_divide(atomic_a, atomic_b)? {
        Atomic::Integer(i) => Ok(Atomic::Integer(i)),
        Atomic::Decimal(d) => Ok(Atomic::Integer(d.trunc().to_i64().unwrap())),
        Atomic::Float(f) => Ok(Atomic::Integer(f.trunc() as i64)),
        Atomic::Double(d) => Ok(Atomic::Integer(d.trunc() as i64)),
        _ => unreachable!(),
    }
}

pub(crate) fn numeric_mod(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    arithmetic_op(
        atomic_a,
        atomic_b,
        ArithmeticOps {
            integer_op: |a, b| {
                if b == 0 {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(a % b)
                }
            },
            decimal_op: |a, b| {
                if b.is_zero() {
                    Err(ValueError::DivisionByZero)
                } else {
                    Ok(a % b)
                }
            },
            float_op: |a, b| a % b,
            double_op: |a, b| a % b,
        },
    )
}

pub(crate) fn numeric_unary_plus(atomic: &Atomic) -> Result<Atomic> {
    match atomic {
        Atomic::Integer(_) => Ok(atomic.clone()),
        Atomic::Decimal(_) => Ok(atomic.clone()),
        Atomic::Float(_) => Ok(atomic.clone()),
        Atomic::Double(_) => Ok(atomic.clone()),
        // XXX function conversion rules?
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn numeric_unary_minus(atomic: &Atomic) -> Result<Atomic> {
    match atomic {
        Atomic::Integer(i) => Ok(Atomic::Integer(-i)),
        Atomic::Decimal(d) => Ok(Atomic::Decimal(-d)),
        Atomic::Float(f) => Ok(Atomic::Float(-f)),
        Atomic::Double(d) => Ok(Atomic::Double(-d)),
        // XXX function conversion rules?
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn numeric_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
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
pub(crate) fn numeric_not_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
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

pub(crate) fn numeric_less_than(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
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

pub(crate) fn numeric_less_than_or_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
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

pub(crate) fn numeric_greater_than(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
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

pub(crate) fn numeric_greater_than_or_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
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

pub(crate) fn string_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::String(a), Atomic::String(b)) => Ok(a == b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn string_not_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::String(a), Atomic::String(b)) => Ok(a != b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn string_less_than(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::String(a), Atomic::String(b)) => Ok(a < b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn string_less_than_or_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::String(a), Atomic::String(b)) => Ok(a <= b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn string_greater_than(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::String(a), Atomic::String(b)) => Ok(a > b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn string_greater_than_or_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::String(a), Atomic::String(b)) => Ok(a >= b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn boolean_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::Boolean(a), Atomic::Boolean(b)) => Ok(a == b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn boolean_not_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::Boolean(a), Atomic::Boolean(b)) => Ok(a != b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn boolean_less_than(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::Boolean(a), Atomic::Boolean(b)) => Ok(a < b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn boolean_less_than_or_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::Boolean(a), Atomic::Boolean(b)) => Ok(a <= b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn boolean_greater_than(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::Boolean(a), Atomic::Boolean(b)) => Ok(a > b),
        _ => Err(ValueError::Type),
    }
}

pub(crate) fn boolean_greater_than_or_equal(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<bool> {
    match (atomic_a, atomic_b) {
        (Atomic::Boolean(a), Atomic::Boolean(b)) => Ok(a >= b),
        _ => Err(ValueError::Type),
    }
}

struct ArithmeticOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>
where
    IntegerOp: FnOnce(i64, i64) -> Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> Result<Decimal>,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> OrderedFloat<f32>,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> OrderedFloat<f64>,
{
    integer_op: IntegerOp,
    decimal_op: DecimalOp,
    float_op: FloatOp,
    double_op: DoubleOp,
}

fn arithmetic_op<IntegerOp, DecimalOp, FloatOp, DoubleOp>(
    atomic_a: &Atomic,
    atomic_b: &Atomic,
    ops: ArithmeticOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>,
) -> Result<Atomic>
where
    IntegerOp: FnOnce(i64, i64) -> Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> Result<Decimal>,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> OrderedFloat<f32>,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> OrderedFloat<f64>,
{
    numeric_general_op(atomic_a, atomic_b, |atomic_a, atomic_b| {
        match (atomic_a, atomic_b) {
            (Atomic::Integer(a), Atomic::Integer(b)) => {
                Ok(Atomic::Integer((ops.integer_op)(*a, *b)?))
            }
            (Atomic::Decimal(a), Atomic::Decimal(b)) => {
                Ok(Atomic::Decimal((ops.decimal_op)(*a, *b)?))
            }
            (Atomic::Float(a), Atomic::Float(b)) => Ok(Atomic::Float((ops.float_op)(*a, *b))),
            (Atomic::Double(a), Atomic::Double(b)) => Ok(Atomic::Double((ops.double_op)(*a, *b))),
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
    atomic_a: &Atomic,
    atomic_b: &Atomic,
    ops: ComparisonOps<IntegerOp, DecimalOp, FloatOp, DoubleOp>,
) -> Result<bool>
where
    IntegerOp: FnOnce(i64, i64) -> bool,
    DecimalOp: FnOnce(Decimal, Decimal) -> bool,
    FloatOp: FnOnce(OrderedFloat<f32>, OrderedFloat<f32>) -> bool,
    DoubleOp: FnOnce(OrderedFloat<f64>, OrderedFloat<f64>) -> bool,
{
    numeric_general_op(atomic_a, atomic_b, |atomic_a, atomic_b| {
        match (atomic_a, atomic_b) {
            (Atomic::Integer(a), Atomic::Integer(b)) => Ok((ops.integer_op)(*a, *b)),
            (Atomic::Decimal(a), Atomic::Decimal(b)) => Ok((ops.decimal_op)(*a, *b)),
            (Atomic::Float(a), Atomic::Float(b)) => Ok((ops.float_op)(*a, *b)),
            (Atomic::Double(a), Atomic::Double(b)) => Ok((ops.double_op)(*a, *b)),
            _ => unreachable!("Illegal combination"),
        }
    })
}

fn numeric_general_op<F, V>(atomic_a: &Atomic, atomic_b: &Atomic, op: F) -> Result<V>
where
    F: FnOnce(&Atomic, &Atomic) -> Result<V>,
{
    // S - type substition due to type hierarchy
    //     https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
    // P - type promotion:
    //    float -> double
    //    decimal -> float
    //    decimal -> double
    match (atomic_a, atomic_b) {
        // -> integer
        (Atomic::Integer(_), Atomic::Integer(_)) => op(atomic_a, atomic_b),
        // -> decimal
        (Atomic::Decimal(_), Atomic::Decimal(_)) => op(atomic_a, atomic_b),
        (Atomic::Integer(_), Atomic::Decimal(_)) => {
            // integer S decimal
            op(&Atomic::Decimal(atomic_a.to_decimal()?), atomic_b)
        }
        (Atomic::Decimal(_), Atomic::Integer(_)) => {
            // integer S decimal
            op(atomic_a, &Atomic::Decimal(atomic_b.to_decimal()?))
        }
        // -> float
        (Atomic::Float(_), Atomic::Float(_)) => op(atomic_a, atomic_b),
        (Atomic::Decimal(_), Atomic::Float(_)) => {
            // decimal P float
            op(&Atomic::Float(atomic_a.to_float()?), atomic_b)
        }
        (Atomic::Integer(_), Atomic::Float(_)) => {
            // integer S decimal P float
            op(&Atomic::Float(atomic_a.to_float()?), atomic_b)
        }
        (Atomic::Float(_), Atomic::Decimal(_)) => {
            // decimal P float
            op(atomic_a, &Atomic::Float(atomic_b.to_float()?))
        }
        (Atomic::Float(_), Atomic::Integer(_)) => {
            // integer S decimal P float
            op(atomic_a, &Atomic::Float(atomic_b.to_float()?))
        }
        // -> double
        (Atomic::Double(_), Atomic::Double(_)) => op(atomic_a, atomic_b),
        (Atomic::Decimal(_), Atomic::Double(_)) => {
            // decimal P double
            op(&Atomic::Double(atomic_a.to_double()?), atomic_b)
        }
        (Atomic::Integer(_), Atomic::Double(_)) => {
            // integer S decimal P double
            op(&Atomic::Double(atomic_a.to_double()?), atomic_b)
        }
        (Atomic::Double(_), Atomic::Decimal(_)) => {
            // decimal P double
            op(atomic_a, &Atomic::Double(atomic_b.to_double()?))
        }
        (Atomic::Double(_), Atomic::Integer(_)) => {
            // integer S decimal P double
            op(atomic_a, &Atomic::Double(atomic_b.to_double()?))
        }
        (Atomic::Float(_), Atomic::Double(_)) => {
            // float P double
            op(&Atomic::Double(atomic_a.to_double()?), atomic_b)
        }
        (Atomic::Double(_), Atomic::Float(_)) => {
            // float P double
            op(atomic_a, &Atomic::Double(atomic_b.to_double()?))
        }
        _ => Err(ValueError::Type),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_add_integers() {
        assert_eq!(
            numeric_add(&Atomic::Integer(1), &Atomic::Integer(2)).unwrap(),
            Atomic::Integer(3)
        );
    }

    #[test]
    fn test_add_integers_overflow() {
        assert_eq!(
            numeric_add(&Atomic::Integer(i64::MAX), &Atomic::Integer(2)),
            Err(ValueError::Overflow)
        );
    }

    #[test]
    fn test_add_decimals() {
        assert_eq!(
            numeric_add(&Atomic::Decimal(dec!(1.5)), &Atomic::Decimal(dec!(2.7))).unwrap(),
            Atomic::Decimal(Decimal::new(42, 1))
        );
    }

    #[test]
    fn test_add_decimals_overflow() {
        assert_eq!(
            numeric_add(&Atomic::Decimal(Decimal::MAX), &Atomic::Decimal(dec!(2.7))),
            Err(ValueError::Overflow)
        );
    }

    #[test]
    fn test_add_floats() {
        assert_eq!(
            numeric_add(
                &Atomic::Float(OrderedFloat(1.5)),
                &Atomic::Float(OrderedFloat(2.7))
            )
            .unwrap(),
            Atomic::Float(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_add_doubles() {
        assert_eq!(
            numeric_add(
                &Atomic::Double(OrderedFloat(1.5)),
                &Atomic::Double(OrderedFloat(2.7))
            )
            .unwrap(),
            Atomic::Double(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_add_integer_decimal() {
        assert_eq!(
            numeric_add(&Atomic::Integer(1), &Atomic::Decimal(dec!(2.7))).unwrap(),
            Atomic::Decimal(Decimal::new(37, 1))
        );
    }

    #[test]
    fn test_add_double_decimal() {
        assert_eq!(
            numeric_add(
                &Atomic::Double(OrderedFloat(1.5)),
                &Atomic::Decimal(dec!(2.7))
            )
            .unwrap(),
            Atomic::Double(OrderedFloat(4.2))
        );
    }

    #[test]
    fn test_numeric_divide_both_integer_returns_decimal() {
        assert_eq!(
            numeric_divide(&Atomic::Integer(1), &Atomic::Integer(2)).unwrap(),
            Atomic::Decimal(dec!(0.5))
        );
    }

    #[test]
    fn test_numeric_integer_divide_10_by_3() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Integer(10), &Atomic::Integer(3)).unwrap(),
            Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_by_minus_2() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Integer(3), &Atomic::Integer(-2)).unwrap(),
            Atomic::Integer(-1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_2() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Integer(-3), &Atomic::Integer(2)).unwrap(),
            Atomic::Integer(-1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_minus_2() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Integer(-3), &Atomic::Integer(-2)).unwrap(),
            Atomic::Integer(1)
        );
    }

    #[test]
    fn test_numeric_integer_divide_9_point_0_by_3() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Double(OrderedFloat(9.0)), &Atomic::Integer(3))
                .unwrap(),
            Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_4() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Double(OrderedFloat(3.0)), &Atomic::Integer(4))
                .unwrap(),
            Atomic::Integer(0)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_by_0() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Integer(3), &Atomic::Integer(0)),
            Err(ValueError::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_0() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Double(OrderedFloat(3.0)), &Atomic::Integer(0)),
            Err(ValueError::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_inf() {
        assert_eq!(
            numeric_integer_divide(
                &Atomic::Double(OrderedFloat(3.0)),
                &Atomic::Double(OrderedFloat(f64::INFINITY))
            )
            .unwrap(),
            Atomic::Integer(0)
        );
    }

    #[test]
    fn test_numeric_mod_nan_nan() {
        assert!(numeric_mod(
            &Atomic::Double(OrderedFloat(f64::NAN)),
            &Atomic::Double(OrderedFloat(f64::NAN))
        )
        .unwrap()
        .is_nan());
    }
}
