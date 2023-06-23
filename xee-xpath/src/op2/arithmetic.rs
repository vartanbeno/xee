// atomized the operand
// if atomized is empty, result is empty
// if atomized is greater than one, type error
// if atomized xs:untypedAtomic, cast to xs:double
// also type substitution, promotion

// so this is like a function that looks ike:

// fn:op($arg1 as xs:numeric, $arg2 as xs:numeric) as xs:numeric

// with an additional untypedAtomic casting rule

// function conversion rules:
// atomization
// xs:untypedAtomic cast to expected type
// numeric items promoted to expectd atomic type
// xs:anyURI promoted too
// also type substitution

use num_traits::{Float, PrimInt};
use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;

use crate::stack;

// type check to see whether it conforms to signature, get out atomized,
// like in the function signature. This takes care of subtype
// relations as that's just a check

// if untypedAtomic, passed through typecheck and cast to double happens
// now do type promotion, conforming the arguments to each other

fn numeric_arithmetic_op<O>(a: stack::Atomic, b: stack::Atomic) -> stack::Result<stack::Atomic>
where
    O: ArithmeticOp,
{
    // we need to extract the values and pass them along now
    match (a, b) {
        (stack::Atomic::Decimal(a), stack::Atomic::Decimal(b)) => {
            <O as ArithmeticOp>::decimal_atomic(a, b)
        }
        (stack::Atomic::Integer(a), stack::Atomic::Integer(b)) => {
            <O as ArithmeticOp>::integer_atomic::<i64>(a, b)
        }
        (stack::Atomic::Int(a), stack::Atomic::Int(b)) => {
            <O as ArithmeticOp>::integer_atomic::<i32>(a, b)
        }
        (stack::Atomic::Short(a), stack::Atomic::Short(b)) => {
            <O as ArithmeticOp>::integer_atomic::<i16>(a, b)
        }
        (stack::Atomic::Byte(a), stack::Atomic::Byte(b)) => {
            <O as ArithmeticOp>::integer_atomic::<i8>(a, b)
        }
        (stack::Atomic::UnsignedLong(a), stack::Atomic::UnsignedLong(b)) => {
            <O as ArithmeticOp>::integer_atomic::<u64>(a, b)
        }
        (stack::Atomic::UnsignedInt(a), stack::Atomic::UnsignedInt(b)) => {
            <O as ArithmeticOp>::integer_atomic::<u32>(a, b)
        }
        (stack::Atomic::UnsignedShort(a), stack::Atomic::UnsignedShort(b)) => {
            <O as ArithmeticOp>::integer_atomic::<u16>(a, b)
        }
        (stack::Atomic::UnsignedByte(a), stack::Atomic::UnsignedByte(b)) => {
            <O as ArithmeticOp>::integer_atomic::<u8>(a, b)
        }
        (stack::Atomic::Float(OrderedFloat(a)), stack::Atomic::Float(OrderedFloat(b))) => {
            <O as ArithmeticOp>::float_atomic::<f32>(a, b)
        }
        (stack::Atomic::Double(OrderedFloat(a)), stack::Atomic::Double(OrderedFloat(b))) => {
            <O as ArithmeticOp>::float_atomic::<f64>(a, b)
        }
        _ => unreachable!("Both the atomics not the same type"),
    }
}

trait ArithmeticOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt;
    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal>;
    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float;

    fn integer_atomic<I>(a: I, b: I) -> stack::Result<stack::Atomic>
    where
        I: PrimInt + Into<stack::Atomic> + Into<Decimal>,
    {
        let v = <Self as ArithmeticOp>::integer(a, b)?;
        Ok(v.into())
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> stack::Result<stack::Atomic> {
        let v = <Self as ArithmeticOp>::decimal(a, b)?;
        Ok(v.into())
    }

    fn float_atomic<F>(a: F, b: F) -> stack::Result<stack::Atomic>
    where
        F: Float + Into<stack::Atomic>,
    {
        let v = <Self as ArithmeticOp>::float(a, b)?;
        Ok(v.into())
    }
}

struct AddOp;

impl ArithmeticOp for AddOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt,
    {
        a.checked_add(&b).ok_or(stack::Error::Overflow)
    }

    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal> {
        a.checked_add(b).ok_or(stack::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float,
    {
        Ok(a + b)
    }
}

struct SubtractOp;

impl ArithmeticOp for SubtractOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt,
    {
        a.checked_sub(&b).ok_or(stack::Error::Overflow)
    }

    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal> {
        a.checked_sub(b).ok_or(stack::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float,
    {
        Ok(a - b)
    }
}

struct MultiplyOp;

impl ArithmeticOp for MultiplyOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt,
    {
        a.checked_mul(&b).ok_or(stack::Error::Overflow)
    }

    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal> {
        a.checked_mul(b).ok_or(stack::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float,
    {
        Ok(a * b)
    }
}

struct DivideOp;

impl ArithmeticOp for DivideOp {
    fn integer_atomic<I>(a: I, b: I) -> stack::Result<stack::Atomic>
    where
        I: PrimInt + Into<stack::Atomic> + Into<Decimal>,
    {
        // As a special case, if the types of both $arg1 and $arg2 are
        // xs:integer, then the return type is xs:decimal.
        let a: Decimal = a.into();
        let b: Decimal = b.into();
        let v = <Self as ArithmeticOp>::decimal(a, b)?;
        Ok(v.into())
    }

    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt,
    {
        if b.is_zero() {
            return Err(stack::Error::DivisionByZero);
        }
        a.checked_div(&b).ok_or(stack::Error::Overflow)
    }

    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal> {
        if b.is_zero() {
            return Err(stack::Error::DivisionByZero);
        }
        a.checked_div(b).ok_or(stack::Error::Overflow)
    }

    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float,
    {
        Ok(a / b)
    }
}

struct NumericIntegerDivideOp;

impl ArithmeticOp for NumericIntegerDivideOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt,
    {
        if b.is_zero() {
            return Err(stack::Error::DivisionByZero);
        }
        a.checked_div(&b).ok_or(stack::Error::Overflow)
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> stack::Result<stack::Atomic> {
        let v = <DivideOp as ArithmeticOp>::decimal(a, b)?;

        Ok(v.trunc().to_i64().ok_or(stack::Error::Overflow)?.into())
    }

    fn decimal(_a: Decimal, _b: Decimal) -> stack::Result<Decimal> {
        unreachable!();
    }

    fn float_atomic<F>(a: F, b: F) -> stack::Result<stack::Atomic>
    where
        F: Float + Into<stack::Atomic>,
    {
        let v = <DivideOp as ArithmeticOp>::float(a, b)?;
        Ok(v.trunc().to_i64().ok_or(stack::Error::Overflow)?.into())
    }

    fn float<F>(_a: F, _b: F) -> stack::Result<F>
    where
        F: Float,
    {
        unreachable!();
    }
}

struct ModOp;

impl ArithmeticOp for ModOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt,
    {
        if b.is_zero() {
            return Err(stack::Error::DivisionByZero);
        }
        Ok(a % b)
    }

    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal> {
        if b.is_zero() {
            return Err(stack::Error::DivisionByZero);
        }
        Ok(a % b)
    }

    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float,
    {
        Ok(a % b)
    }
}

fn numeric_unary_plus(atomic: stack::Atomic) -> stack::Result<stack::Atomic> {
    if atomic.is_numeric() {
        Ok(atomic)
    } else {
        Err(stack::Error::Type)
    }
}

fn numeric_unary_minus(atomic: stack::Atomic) -> stack::Result<stack::Atomic> {
    if atomic.is_numeric() {
        match atomic {
            stack::Atomic::Decimal(v) => Ok(stack::Atomic::Decimal(-v)),
            stack::Atomic::Integer(v) => Ok(stack::Atomic::Integer(-v)),
            stack::Atomic::Int(v) => Ok(stack::Atomic::Int(-v)),
            stack::Atomic::Short(v) => Ok(stack::Atomic::Short(-v)),
            stack::Atomic::Byte(v) => Ok(stack::Atomic::Byte(-v)),
            stack::Atomic::Float(v) => Ok(stack::Atomic::Float(-v)),
            stack::Atomic::Double(v) => Ok(stack::Atomic::Double(-v)),
            // what is the correct behavior for unsigned types? We could return
            // a signed integer of the same type with overflow behavior if
            // that's not possible, but for now we just refuse to do it.
            stack::Atomic::UnsignedLong(_) => Err(stack::Error::Type),
            stack::Atomic::UnsignedInt(_) => Err(stack::Error::Type),
            stack::Atomic::UnsignedShort(_) => Err(stack::Error::Type),
            stack::Atomic::UnsignedByte(_) => Err(stack::Error::Type),
            _ => unreachable!(),
        }
    } else {
        Err(stack::Error::Type)
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
        let result = numeric_arithmetic_op::<AddOp>(a, b).unwrap();
        assert_eq!(result, 3i64.into());
    }

    #[test]
    fn test_integer_division_returns_decimal() {
        let a = 5i64.into();
        let b = 2i64.into();
        let result = numeric_arithmetic_op::<DivideOp>(a, b).unwrap();
        assert_eq!(result, dec!(2.5).into());
    }

    #[test]
    fn test_numeric_integer_divide() {
        let a = 5i64.into();
        let b = 2i64.into();
        let result = numeric_arithmetic_op::<NumericIntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 2i64.into());
    }

    #[test]
    fn test_numeric_integer_divide_float() {
        let a = 5f64.into();
        let b = 2f64.into();
        let result = numeric_arithmetic_op::<NumericIntegerDivideOp>(a, b).unwrap();
        assert_eq!(result, 2i64.into());
    }
}
