use rust_decimal::prelude::*;

use crate::value::{Atomic, ValueError};

type Result<T> = std::result::Result<T, ValueError>;

fn numeric_add(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    numeric_op(
        atomic_a,
        atomic_b,
        Ops {
            integer_op: |a, b| a.checked_add(b).ok_or(ValueError::Overflow),
            decimal_op: |a, b| a.checked_add(b).ok_or(ValueError::Overflow),
            float_op: |a, b| a + b,
            double_op: |a, b| a + b,
        },
    )
}

fn numeric_substract(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    numeric_op(
        atomic_a,
        atomic_b,
        Ops {
            integer_op: |a, b| a.checked_sub(b).ok_or(ValueError::Overflow),
            decimal_op: |a, b| a.checked_sub(b).ok_or(ValueError::Overflow),
            float_op: |a, b| a - b,
            double_op: |a, b| a - b,
        },
    )
}

fn numeric_multiply(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    numeric_op(
        atomic_a,
        atomic_b,
        Ops {
            integer_op: |a, b| a.checked_mul(b).ok_or(ValueError::Overflow),
            decimal_op: |a, b| a.checked_mul(b).ok_or(ValueError::Overflow),
            float_op: |a, b| a * b,
            double_op: |a, b| a * b,
        },
    )
}

fn numeric_divide(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    match (atomic_a, atomic_b) {
        // As a special case, if the types of both $arg1 and $arg2 are
        // xs:integer, then the return type is xs:decimal.
        (Atomic::Integer(_), Atomic::Integer(_)) => numeric_divide(
            &Atomic::Decimal(atomic_a.as_decimal().unwrap()),
            &Atomic::Decimal(atomic_b.as_decimal().unwrap()),
        ),
        _ => {
            numeric_op(
                atomic_a,
                atomic_b,
                Ops {
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

fn numeric_integer_divide(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
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

struct Ops<IntegerOp, DecimalOp, FloatOp, DoubleOp>
where
    IntegerOp: FnOnce(i64, i64) -> Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> Result<Decimal>,
    FloatOp: FnOnce(f32, f32) -> f32,
    DoubleOp: FnOnce(f64, f64) -> f64,
{
    integer_op: IntegerOp,
    decimal_op: DecimalOp,
    float_op: FloatOp,
    double_op: DoubleOp,
}

fn numeric_op<IntegerOp, DecimalOp, FloatOp, DoubleOp>(
    atomic_a: &Atomic,
    atomic_b: &Atomic,
    ops: Ops<IntegerOp, DecimalOp, FloatOp, DoubleOp>,
) -> Result<Atomic>
where
    IntegerOp: FnOnce(i64, i64) -> Result<i64>,
    DecimalOp: FnOnce(Decimal, Decimal) -> Result<Decimal>,
    FloatOp: FnOnce(f32, f32) -> f32,
    DoubleOp: FnOnce(f64, f64) -> f64,
{
    // S - type substition due to type hierarchy
    //     https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy
    // P - type promotion:
    //    float -> double
    //    decimal -> float
    //    decimal -> double
    match (atomic_a, atomic_b) {
        // -> integer
        (Atomic::Integer(a), Atomic::Integer(b)) => Ok(Atomic::Integer((ops.integer_op)(*a, *b)?)),
        // -> decimal
        (Atomic::Decimal(a), Atomic::Decimal(b)) => Ok(Atomic::Decimal((ops.decimal_op)(*a, *b)?)),
        (Atomic::Integer(_), Atomic::Decimal(_)) => {
            // integer S decimal
            numeric_op(&Atomic::Decimal(atomic_a.as_decimal()?), atomic_b, ops)
        }
        (Atomic::Decimal(_), Atomic::Integer(_)) => {
            // integer S decimal
            numeric_op(atomic_a, &Atomic::Decimal(atomic_b.as_decimal()?), ops)
        }
        // -> float
        (Atomic::Float(a), Atomic::Float(b)) => Ok(Atomic::Float((ops.float_op)(*a, *b))),
        (Atomic::Decimal(_), Atomic::Float(_)) => {
            // decimal P float
            numeric_op(&Atomic::Float(atomic_a.as_float()?), atomic_b, ops)
        }
        (Atomic::Integer(_), Atomic::Float(_)) => {
            // integer S decimal P float
            numeric_op(&Atomic::Float(atomic_a.as_float()?), atomic_b, ops)
        }
        (Atomic::Float(_), Atomic::Decimal(_)) => {
            // decimal P float
            numeric_op(atomic_a, &Atomic::Float(atomic_b.as_float()?), ops)
        }
        (Atomic::Float(_), Atomic::Integer(_)) => {
            // integer S decimal P float
            numeric_op(atomic_a, &Atomic::Float(atomic_b.as_float()?), ops)
        }
        // -> double
        (Atomic::Double(a), Atomic::Double(b)) => Ok(Atomic::Double((ops.double_op)(*a, *b))),
        (Atomic::Decimal(_), Atomic::Double(_)) => {
            // decimal P double
            numeric_op(&Atomic::Double(atomic_a.as_double()?), atomic_b, ops)
        }
        (Atomic::Integer(_), Atomic::Double(_)) => {
            // integer S decimal P double
            numeric_op(&Atomic::Double(atomic_a.as_double()?), atomic_b, ops)
        }
        (Atomic::Double(_), Atomic::Decimal(_)) => {
            // decimal P double
            numeric_op(atomic_a, &Atomic::Double(atomic_b.as_double()?), ops)
        }
        (Atomic::Double(_), Atomic::Integer(_)) => {
            // integer S decimal P double
            numeric_op(atomic_a, &Atomic::Double(atomic_b.as_double()?), ops)
        }
        (Atomic::Float(_), Atomic::Double(_)) => {
            // float P double
            numeric_op(&Atomic::Double(atomic_a.as_double()?), atomic_b, ops)
        }
        (Atomic::Double(_), Atomic::Float(_)) => {
            // float P double
            numeric_op(atomic_a, &Atomic::Double(atomic_b.as_double()?), ops)
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
            numeric_add(&Atomic::Float(1.5), &Atomic::Float(2.7)).unwrap(),
            Atomic::Float(4.2)
        );
    }

    #[test]
    fn test_add_doubles() {
        assert_eq!(
            numeric_add(&Atomic::Double(1.5), &Atomic::Double(2.7)).unwrap(),
            Atomic::Double(4.2)
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
            numeric_add(&Atomic::Double(1.5), &Atomic::Decimal(dec!(2.7))).unwrap(),
            Atomic::Double(4.2)
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
            numeric_integer_divide(&Atomic::Double(9.0), &Atomic::Integer(3)).unwrap(),
            Atomic::Integer(3)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_4() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Double(3.0), &Atomic::Integer(4)).unwrap(),
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
            numeric_integer_divide(&Atomic::Double(3.0), &Atomic::Integer(0)),
            Err(ValueError::DivisionByZero)
        );
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_inf() {
        assert_eq!(
            numeric_integer_divide(&Atomic::Double(3.0), &Atomic::Double(f64::INFINITY)).unwrap(),
            Atomic::Integer(0)
        );
    }
}
