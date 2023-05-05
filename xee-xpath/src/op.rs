use rust_decimal::prelude::*;

use crate::value::{Atomic, ValueError};

type Result<T> = std::result::Result<T, ValueError>;

fn numeric_add(atomic_a: &Atomic, atomic_b: &Atomic) -> Result<Atomic> {
    numeric_op(
        atomic_a,
        atomic_b,
        Ops {
            integer_op: |a, b| a.checked_add(b).ok_or(ValueError::OverflowError),
            decimal_op: |a, b| a.checked_add(b).ok_or(ValueError::OverflowError),
            float_op: |a, b| a + b,
            double_op: |a, b| a + b,
        },
    )
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
        _ => Err(ValueError::TypeError),
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
            Err(ValueError::OverflowError)
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
            Err(ValueError::OverflowError)
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
}
