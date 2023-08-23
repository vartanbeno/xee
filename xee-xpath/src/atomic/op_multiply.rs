use std::rc::Rc;

use ibig::IBig;
use ordered_float::OrderedFloat;
use rust_decimal::Decimal;

use crate::atomic;
use crate::error;

use super::cast_numeric::cast_numeric;

use super::cast_numeric::f64_to_i64;
use super::datetime::YearMonthDuration;
use super::types::IntegerType;

pub(crate) fn op_multiply(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<atomic::Atomic> {
    use atomic::Atomic;

    let (a, b) = cast_numeric(a, b)?;

    match (a, b) {
        (Atomic::Decimal(a), Atomic::Decimal(b)) => Ok(Atomic::Decimal(op_multiply_decimal(a, b)?)),
        (Atomic::Integer(_, a), Atomic::Integer(_, b)) => Ok(Atomic::Integer(
            IntegerType::Integer,
            op_multiply_integer(a, b),
        )),
        (Atomic::Float(a), Atomic::Float(b)) => Ok(Atomic::Float(a * b)),
        (Atomic::Double(a), Atomic::Double(b)) => Ok(Atomic::Double(a * b)),
        //  op:multiply-yearMonthDuration(A, B) -> xs:yearMonthDuration
        (Atomic::YearMonthDuration(a), b @ Atomic::Decimal(_))
        | (b @ Atomic::Decimal(_), Atomic::YearMonthDuration(a)) => Ok(Atomic::YearMonthDuration(
            op_multiply_year_month_duration_by_atomic(a, b)?,
        )),
        (Atomic::YearMonthDuration(a), b @ Atomic::Integer(_, _))
        | (b @ Atomic::Integer(_, _), Atomic::YearMonthDuration(a)) => Ok(
            Atomic::YearMonthDuration(op_multiply_year_month_duration_by_atomic(a, b)?),
        ),
        (Atomic::YearMonthDuration(a), b @ Atomic::Float(_))
        | (b @ Atomic::Float(_), Atomic::YearMonthDuration(a)) => Ok(Atomic::YearMonthDuration(
            op_multiply_year_month_duration_by_atomic(a, b)?,
        )),
        (Atomic::YearMonthDuration(a), Atomic::Double(OrderedFloat(b)))
        | (Atomic::Double(OrderedFloat(b)), Atomic::YearMonthDuration(a)) => Ok(
            Atomic::YearMonthDuration(op_multiply_year_month_duration_by_double(a, b)?),
        ),
        // op:multiply-dayTimeDuration(A, B) -> xs:dayTimeDuration
        (Atomic::DayTimeDuration(a), b @ Atomic::Decimal(_))
        | (b @ Atomic::Decimal(_), Atomic::DayTimeDuration(a)) => Ok(Atomic::DayTimeDuration(
            op_multiply_day_time_duration_by_atomic(a, b)?,
        )),
        (Atomic::DayTimeDuration(a), b @ Atomic::Integer(_, _))
        | (b @ Atomic::Integer(_, _), Atomic::DayTimeDuration(a)) => Ok(Atomic::DayTimeDuration(
            op_multiply_day_time_duration_by_atomic(a, b)?,
        )),
        (Atomic::DayTimeDuration(a), b @ Atomic::Float(_))
        | (b @ Atomic::Float(_), Atomic::DayTimeDuration(a)) => Ok(Atomic::DayTimeDuration(
            op_multiply_day_time_duration_by_atomic(a, b)?,
        )),
        (Atomic::DayTimeDuration(a), Atomic::Double(OrderedFloat(b)))
        | (Atomic::Double(OrderedFloat(b)), Atomic::DayTimeDuration(a)) => Ok(
            Atomic::DayTimeDuration(op_multiply_day_time_duration_by_double(a, b)?),
        ),
        _ => Err(error::Error::Type),
    }
}

fn op_multiply_decimal(a: Rc<Decimal>, b: Rc<Decimal>) -> error::Result<Rc<Decimal>> {
    Ok(Rc::new(
        a.as_ref()
            .checked_mul(*b.as_ref())
            .ok_or(error::Error::Overflow)?,
    ))
}

fn op_multiply_integer(a: Rc<IBig>, b: Rc<IBig>) -> Rc<IBig> {
    Rc::new(a.as_ref() * b.as_ref())
}

fn op_multiply_year_month_duration_by_atomic(
    a: YearMonthDuration,
    b: atomic::Atomic,
) -> error::Result<YearMonthDuration> {
    let b = b.cast_to_double()?;
    let b = match b {
        atomic::Atomic::Double(OrderedFloat(b)) => b,
        _ => unreachable!(),
    };
    op_multiply_year_month_duration_by_double(a, b)
}

fn op_multiply_year_month_duration_by_double(
    a: YearMonthDuration,
    b: f64,
) -> error::Result<YearMonthDuration> {
    if b.is_nan() {
        return Err(error::Error::FOCA0005);
    }
    let total = f64_to_i64(a.months as f64 * b)?;
    Ok(YearMonthDuration::new(total))
}

fn op_multiply_day_time_duration_by_atomic(
    a: Rc<chrono::Duration>,
    b: atomic::Atomic,
) -> error::Result<Rc<chrono::Duration>> {
    let b = b.cast_to_double()?;
    let b = match b {
        atomic::Atomic::Double(OrderedFloat(b)) => b,
        _ => unreachable!(),
    };
    op_multiply_day_time_duration_by_double(a, b)
}

fn op_multiply_day_time_duration_by_double(
    a: Rc<chrono::Duration>,
    b: f64,
) -> error::Result<Rc<chrono::Duration>> {
    if b.is_nan() {
        return Err(error::Error::FOCA0005);
    }
    let a = a.num_milliseconds() as f64;
    let total = f64_to_i64(a * b)?;
    Ok(Rc::new(chrono::Duration::milliseconds(total)))
}
