use ibig::IBig;
use num_traits::Float;
use rust_decimal::Decimal;
use std::rc::Rc;

use crate::atomic;
use crate::error;

use super::cast_numeric::cast_numeric;

pub(crate) fn op_div(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<atomic::Atomic> {
    use atomic::Atomic;

    let (a, b) = cast_numeric(a, b)?;

    match (a, b) {
        (Atomic::Decimal(a), Atomic::Decimal(b)) => {
            Ok(Atomic::Decimal(Rc::new(op_div_decimal(a, b)?)))
        }
        (Atomic::Integer(_, a), Atomic::Integer(_, b)) => Ok(op_div_integer(a, b)?),
        (Atomic::Float(a), Atomic::Float(b)) => Ok(Atomic::Float(op_div_float(a, b)?)),
        (Atomic::Double(a), Atomic::Double(b)) => Ok(Atomic::Double(op_div_float(a, b)?)),
        _ => Err(error::Error::Type),
    }
}

pub(crate) fn op_div_decimal(a: Rc<Decimal>, b: Rc<Decimal>) -> error::Result<Decimal> {
    if b.is_zero() {
        return Err(error::Error::DivisionByZero);
    }
    a.checked_div(*b.as_ref()).ok_or(error::Error::Overflow)
}

fn op_div_integer(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<atomic::Atomic> {
    // As a special case, if the types of both $arg1 and $arg2 are
    // xs:integer, then the return type is xs:decimal.
    let a: i128 = a.as_ref().try_into().map_err(|_| error::Error::FOCA0001)?;
    let b: i128 = b.as_ref().try_into().map_err(|_| error::Error::FOCA0001)?;
    let v = op_div_decimal(Rc::new(a.into()), Rc::new(b.into()))?;
    Ok(v.into())
}

pub(crate) fn op_div_float<F>(a: F, b: F) -> error::Result<F>
where
    F: Float,
{
    if b.is_zero() {
        return Err(error::Error::DivisionByZero);
    }
    Ok(a / b)
}
