// This op is like a function with this argument:

// fn:op($arg1 as xs:numeric, $arg2 as xs:numeric) as xs:numeric

// with an additional untypedAtomic casting rule

use ibig::IBig;
use num_traits::Float;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::rc::Rc;
use xee_schema_type::BaseNumericType;

use crate::atomic;
use crate::error;

use super::cast::cast_numeric_binary;

pub(crate) fn arithmetic_op<O>(
    a: atomic::Atomic,
    b: atomic::Atomic,
) -> error::Result<atomic::Atomic>
where
    O: ArithmeticOp,
{
    let (a, b) = cast(a, b)?;
    // we need to extract the values and pass them along now
    match (a, b) {
        (atomic::Atomic::Decimal(a), atomic::Atomic::Decimal(b)) => {
            <O as ArithmeticOp>::decimal_atomic(a, b)
        }
        (atomic::Atomic::Integer(a), atomic::Atomic::Integer(b)) => {
            <O as ArithmeticOp>::ibig_atomic(a, b)
        }
        (atomic::Atomic::Float(OrderedFloat(a)), atomic::Atomic::Float(OrderedFloat(b))) => {
            <O as ArithmeticOp>::float_atomic::<f32>(a, b)
        }
        (atomic::Atomic::Double(OrderedFloat(a)), atomic::Atomic::Double(OrderedFloat(b))) => {
            <O as ArithmeticOp>::float_atomic::<f64>(a, b)
        }
        _ => unreachable!("Both the atomics not the same type"),
    }
}

fn cast(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<(atomic::Atomic, atomic::Atomic)> {
    // 3.5 arithmetic expressions
    // https://www.w3.org/TR/xpath-31/#id-arithmetic

    // We start in step 4, as the previous steps have been handled
    // by the caller.

    // 4: If an atomized operand of of type xs:untypedAtomic, it is cast
    // to xs:double
    let a = cast_untyped(a)?;
    let b = cast_untyped(b)?;

    cast_numeric_binary(a, b, |_a, _b| Err(error::Error::Type))
}

fn cast_untyped(value: atomic::Atomic) -> error::Result<atomic::Atomic> {
    if let atomic::Atomic::Untyped(s) = value {
        atomic::Atomic::parse_atomic::<f64>(&s)
    } else {
        Ok(value)
    }
}

pub(crate) trait ArithmeticOp {
    // fn integer<I>(a: I, b: I) -> error::Result<I>
    // where
    //     I: PrimInt;
    fn ibig(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>>;
    fn decimal(a: Decimal, b: Decimal) -> error::Result<Decimal>;
    fn float<F>(a: F, b: F) -> error::Result<F>
    where
        F: Float;

    fn ibig_atomic(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<atomic::Atomic> {
        let v = <Self as ArithmeticOp>::ibig(a, b)?;
        Ok(v.into())
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> error::Result<atomic::Atomic> {
        let v = <Self as ArithmeticOp>::decimal(a, b)?;
        Ok(v.into())
    }

    fn float_atomic<F>(a: F, b: F) -> error::Result<atomic::Atomic>
    where
        F: Float + Into<atomic::Atomic>,
    {
        let v = <Self as ArithmeticOp>::float(a, b)?;
        Ok(v.into())
    }
}

pub(crate) struct AddOp;

impl ArithmeticOp for AddOp {
    fn ibig(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>> {
        Ok(Rc::new(a.as_ref() + b.as_ref()))
    }

    fn decimal(a: Decimal, b: Decimal) -> error::Result<Decimal> {
        a.checked_add(b).ok_or(error::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> error::Result<F>
    where
        F: Float,
    {
        Ok(a + b)
    }
}

pub(crate) struct SubtractOp;

impl ArithmeticOp for SubtractOp {
    fn ibig(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>> {
        Ok(Rc::new(a.as_ref() - b.as_ref()))
    }

    fn decimal(a: Decimal, b: Decimal) -> error::Result<Decimal> {
        a.checked_sub(b).ok_or(error::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> error::Result<F>
    where
        F: Float,
    {
        Ok(a - b)
    }
}

pub(crate) struct MultiplyOp;

impl ArithmeticOp for MultiplyOp {
    fn ibig(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>> {
        Ok(Rc::new(a.as_ref() * b.as_ref()))
    }

    fn decimal(a: Decimal, b: Decimal) -> error::Result<Decimal> {
        a.checked_mul(b).ok_or(error::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> error::Result<F>
    where
        F: Float,
    {
        Ok(a * b)
    }
}

pub(crate) struct DivideOp;

impl ArithmeticOp for DivideOp {
    fn ibig_atomic(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<atomic::Atomic> {
        // As a special case, if the types of both $arg1 and $arg2 are
        // xs:integer, then the return type is xs:decimal.
        let a: i128 = a.as_ref().try_into().map_err(|_| error::Error::FOCA0001)?;
        let b: i128 = b.as_ref().try_into().map_err(|_| error::Error::FOCA0001)?;
        let v = <Self as ArithmeticOp>::decimal(a.into(), b.into())?;
        Ok(v.into())
    }

    fn ibig(_a: Rc<IBig>, _b: Rc<IBig>) -> error::Result<Rc<IBig>> {
        unreachable!()
    }

    fn decimal(a: Decimal, b: Decimal) -> error::Result<Decimal> {
        if b.is_zero() {
            return Err(error::Error::DivisionByZero);
        }
        a.checked_div(b).ok_or(error::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> error::Result<F>
    where
        F: Float,
    {
        Ok(a / b)
    }
}

pub(crate) struct IntegerDivideOp;

impl ArithmeticOp for IntegerDivideOp {
    fn ibig(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>> {
        if b.is_zero() {
            return Err(error::Error::DivisionByZero);
        }
        Ok(Rc::new(a.as_ref() / b.as_ref()))
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> error::Result<atomic::Atomic> {
        let v = <DivideOp as ArithmeticOp>::decimal(a, b)?;

        let v: i128 = v.trunc().to_i128().ok_or(error::Error::Overflow)?;
        let i: IBig = v.try_into().map_err(|_| error::Error::Overflow)?;
        Ok(i.into())
    }

    fn decimal(_a: Decimal, _b: Decimal) -> error::Result<Decimal> {
        unreachable!();
    }

    fn float_atomic<F>(a: F, b: F) -> error::Result<atomic::Atomic>
    where
        F: Float + Into<atomic::Atomic>,
    {
        if b.is_zero() {
            return Err(error::Error::DivisionByZero);
        }
        let v = <DivideOp as ArithmeticOp>::float(a, b)?;
        let v: i128 = v.trunc().to_i128().ok_or(error::Error::Overflow)?;
        let i: IBig = v.try_into().map_err(|_| error::Error::Overflow)?;
        Ok(i.into())
    }

    fn float<F>(_a: F, _b: F) -> error::Result<F>
    where
        F: Float,
    {
        unreachable!();
    }
}

pub(crate) struct ModuloOp;

impl ArithmeticOp for ModuloOp {
    fn ibig(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>> {
        if b.is_zero() {
            return Err(error::Error::DivisionByZero);
        }
        Ok(Rc::new(a.as_ref() % b.as_ref()))
    }

    fn decimal(a: Decimal, b: Decimal) -> error::Result<Decimal> {
        if b.is_zero() {
            return Err(error::Error::DivisionByZero);
        }
        Ok(a % b)
    }

    fn float<F>(a: F, b: F) -> error::Result<F>
    where
        F: Float,
    {
        Ok(a % b)
    }
}

pub(crate) fn unary_plus(atomic: atomic::Atomic) -> error::Result<atomic::Atomic> {
    let schema_type = atomic.schema_type();
    let numeric_type = schema_type.base_numeric_type();

    use BaseNumericType::*;
    match numeric_type {
        Some(Integer) => atomic.cast_to_integer(),
        Some(Decimal) => atomic.cast_to_decimal(),
        Some(Float) => atomic.cast_to_float(),
        Some(Double) => atomic.cast_to_double(),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn unary_minus(atomic: atomic::Atomic) -> error::Result<atomic::Atomic> {
    let schema_type = atomic.schema_type();
    let numeric_type = schema_type.base_numeric_type();

    use BaseNumericType::*;
    match numeric_type {
        Some(Integer) => {
            let v: IBig = atomic.cast_to_integer()?.try_into()?;
            Ok(atomic::Atomic::Integer((-v).into()))
        }
        Some(Decimal) => {
            let v: rust_decimal::Decimal = atomic.cast_to_decimal()?.try_into()?;
            Ok(atomic::Atomic::Decimal(-v))
        }
        Some(Float) => {
            let v: f32 = atomic.cast_to_float()?.try_into()?;
            Ok(atomic::Atomic::Float(OrderedFloat(-v)))
        }
        Some(Double) => {
            let v: f64 = atomic.cast_to_double()?.try_into()?;
            Ok(atomic::Atomic::Double(OrderedFloat(-v)))
        }
        _ => Err(error::Error::Type),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;

    #[test]
    fn test_add_ints() {
        let a = 1i64.into();
        let b = 2i64.into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, 3i64.into());
    }

    #[test]
    fn test_integer_division_returns_decimal() {
        let a = 5i64.into();
        let b = 2i64.into();
        let result = arithmetic_op::<DivideOp>(a, b).unwrap();
        assert_eq!(result, dec!(2.5).into());
    }

    #[test]
    fn test_numeric_integer_divide() {
        let a = 5i64.into();
        let b = 2i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 2i64.into());
    }

    #[test]
    fn test_numeric_integer_divide_float() {
        let a = 5f64.into();
        let b = 2f64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 2i64.into());
    }

    #[test]
    fn test_add_integers() {
        let a = 1i64.into();
        let b = 2i64.into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, 3i64.into());
    }

    #[test]
    fn test_add_decimals() {
        let a = dec!(1.5).into();
        let b = dec!(2.7).into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, dec!(4.2).into());
    }

    #[test]
    fn test_add_decimals_overflow() {
        let a = Decimal::MAX.into();
        let b = dec!(2.7).into();
        let result = arithmetic_op::<AddOp>(a, b);
        assert_eq!(result, Err(error::Error::Overflow));
    }

    #[test]
    fn test_add_floats() {
        let a = 1.5f32.into();
        let b = 2.7f32.into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, 4.2f32.into());
    }

    #[test]
    fn test_add_doubles() {
        let a = 1.5f64.into();
        let b = 2.7f64.into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, 4.2f64.into());
    }

    #[test]
    fn test_add_integer_decimal() {
        let a = 1i64.into();
        let b = dec!(2.7).into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, dec!(3.7).into());
    }

    #[test]
    fn test_add_double_decimal() {
        let a = 1.5f64.into();
        let b = dec!(2.7).into();
        let result = arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, dec!(4.2).into());
    }

    #[test]
    fn test_numeric_divide_both_integer_returns_decimal() {
        let a = 1i64.into();
        let b = 2i64.into();
        let result = arithmetic_op::<DivideOp>(a, b).unwrap();
        assert_eq!(result, dec!(0.5).into());
    }

    #[test]
    fn test_numeric_integer_divide_10_by_3() {
        let a = 10i64.into();
        let b = 3i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 3i64.into());
    }

    #[test]
    fn test_numeric_integer_divide_3_by_minus_2() {
        let a = 3i64.into();
        let b = (-2i64).into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, (-1i64).into());
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_2() {
        let a = (-3i64).into();
        let b = 2i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, (-1i64).into());
    }

    #[test]
    fn test_numeric_integer_divide_minus_3_by_minus_2() {
        let a = (-3i64).into();
        let b = (-2i64).into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 1i64.into());
    }

    #[test]
    fn test_numeric_integer_divide_9_point_0_by_3() {
        let a = 9.0f64.into();
        let b = 3i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 3i64.into());
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_4() {
        let a = 3.0f32.into();
        let b = 4i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 0i64.into());
    }

    #[test]
    fn test_numeric_integer_divide_3_by_0() {
        let a = 3i64.into();
        let b = 0i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b);
        assert_eq!(result, Err(error::Error::DivisionByZero));
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_0() {
        let a = 3.0f64.into();
        let b = 0i64.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b);
        assert_eq!(result, Err(error::Error::DivisionByZero));
    }

    #[test]
    fn test_numeric_integer_divide_3_point_0_by_inf() {
        let a = 3.0f64.into();
        let b = f64::INFINITY.into();
        let result = arithmetic_op::<IntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 0i64.into());
    }

    #[test]
    fn test_numeric_mod_nan_nan() {
        let a = f64::NAN.into();
        let b = f64::NAN.into();
        let result = arithmetic_op::<ModuloOp>(a, b).unwrap();
        assert!(result.is_nan());
    }
}
