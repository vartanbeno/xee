use ibig::IBig;
use num_traits::Float;
use num_traits::ToPrimitive;
use num_traits::Zero;
use rust_decimal::Decimal;
use std::rc::Rc;

use crate::atomic;
use crate::error;

use super::cast_numeric::cast_numeric;
use super::op_div::{op_div_decimal, op_div_float};
use super::types::IntegerType;

pub(crate) fn op_idiv(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<atomic::Atomic> {
    use atomic::Atomic;

    let (a, b) = cast_numeric(a, b)?;

    match (a, b) {
        (Atomic::Decimal(a), Atomic::Decimal(b)) => Ok(op_idiv_decimal(a, b)?),
        (Atomic::Integer(_, a), Atomic::Integer(_, b)) => Ok(Atomic::Integer(
            IntegerType::Integer,
            op_idiv_integer(a, b)?,
        )),
        (Atomic::Float(a), Atomic::Float(b)) => Ok(op_idiv_float(a, b)?),
        (Atomic::Double(a), Atomic::Double(b)) => Ok(op_idiv_float(a, b)?),
        _ => Err(error::Error::Type),
    }
}

fn op_idiv_decimal(a: Rc<Decimal>, b: Rc<Decimal>) -> error::Result<atomic::Atomic> {
    let v = op_div_decimal(a, b)?;
    let v: i128 = v.trunc().to_i128().ok_or(error::Error::Overflow)?;
    let i: IBig = v.try_into().map_err(|_| error::Error::Overflow)?;
    Ok(i.into())
}

fn op_idiv_integer(a: Rc<IBig>, b: Rc<IBig>) -> error::Result<Rc<IBig>> {
    if b.is_zero() {
        return Err(error::Error::DivisionByZero);
    }
    Ok(Rc::new(a.as_ref() / b.as_ref()))
}

fn op_idiv_float<F>(a: F, b: F) -> error::Result<atomic::Atomic>
where
    F: Float + Into<atomic::Atomic>,
{
    if b.is_zero() {
        return Err(error::Error::DivisionByZero);
    }
    let v = op_div_float(a, b)?;
    let v: i128 = v.trunc().to_i128().ok_or(error::Error::Overflow)?;
    let i: IBig = v.try_into().map_err(|_| error::Error::Overflow)?;
    Ok(i.into())
}
