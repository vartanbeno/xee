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
use rust_decimal::Decimal;

use crate::error;
use crate::output;
use crate::stack;

// type check to see whether it conforms to signature, get out atomized,
// like in the function signature. This takes care of subtype
// relations as that's just a check

// if untypedAtomic, passed through typecheck and cast to double happens
// now do type promotion, conforming the arguments to each other

fn numeric_op<O>(a: output::Atomic, b: output::Atomic) -> error::Result<output::Sequence>
where
    O: ArithmeticOp,
{
    // we need to extract the values and pass them along now
    let value = match (a.stack_atomic, b.stack_atomic) {
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
    }?;
    let item = output::Item::from(value);
    let sequence = output::Sequence::from(item);
    Ok(sequence)
}

trait ArithmeticOp {
    fn integer<I>(a: I, b: I) -> stack::Result<I>
    where
        I: PrimInt;
    fn decimal(a: Decimal, b: Decimal) -> stack::Result<Decimal>;
    fn float<F>(a: F, b: F) -> stack::Result<F>
    where
        F: Float;

    fn integer_atomic<I>(a: I, b: I) -> error::Result<output::Atomic>
    where
        I: PrimInt + Into<output::Atomic>,
    {
        let v = <Self as ArithmeticOp>::integer(a, b);
        v.map(|v| v.into()).map_err(|e| e.into())
    }

    fn decimal_atomic(a: Decimal, b: Decimal) -> error::Result<output::Atomic> {
        let v = <Self as ArithmeticOp>::decimal(a, b);
        v.map(|v| v.into()).map_err(|e| e.into())
    }

    fn float_atomic<F>(a: F, b: F) -> error::Result<output::Atomic>
    where
        F: Float + Into<output::Atomic>,
    {
        let v = <Self as ArithmeticOp>::float(a, b);
        v.map(|v| v.into()).map_err(|e| e.into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_ints() {
        let a = 1i64.into();
        let b = 2i64.into();
        let result = numeric_op::<AddOp>(a, b).unwrap();
        let result = result.items().next().unwrap();
        assert_eq!(result, 3i64.into());
    }
}
