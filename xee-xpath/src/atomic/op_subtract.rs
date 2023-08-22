use std::rc::Rc;

use ibig::IBig;
use rust_decimal::Decimal;

use crate::atomic;
use crate::error;

use super::cast_numeric::cast_numeric;
use super::datetime::{
    NaiveDateTimeWithOffset, NaiveDateWithOffset, NaiveTimeWithOffset, YearMonthDuration,
};
use super::types::IntegerType;

pub(crate) fn op_subtract(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<atomic::Atomic> {
    use atomic::Atomic;

    let (a, b) = cast_numeric(a, b)?;

    match (a, b) {
        (Atomic::Decimal(a), Atomic::Decimal(b)) => {
            Ok(Atomic::Decimal(op_substract_decimal(a, b)?))
        }
        (Atomic::Integer(_, a), Atomic::Integer(_, b)) => Ok(Atomic::Integer(
            IntegerType::Integer,
            op_substract_integer(a, b),
        )),
        (Atomic::Float(a), Atomic::Float(b)) => Ok(Atomic::Float(a - b)),
        (Atomic::Double(a), Atomic::Double(b)) => Ok(Atomic::Double(a - b)),
        // op:subtract-yearMonthDuration-from-date(A, B) -> xs:date
        (Atomic::Date(a), Atomic::YearMonthDuration(b))
        | (Atomic::YearMonthDuration(b), Atomic::Date(a)) => Ok(Atomic::Date(
            op_subtract_year_month_duration_from_date(a, b)?,
        )),
        // op:subtract-dayTimeDuration-from-date(A, B) -> xs:date
        (Atomic::Date(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::Date(a)) => Ok(Atomic::Date(
            op_subtract_day_time_duration_from_date(a, *b)?,
        )),
        // op:subtract-times(A, B) -> xs:dayTimeDuration
        (Atomic::Time(a), Atomic::Time(b)) => Ok(Atomic::DayTimeDuration(op_subtract_times(a, b)?)),
        // op:subtract-dayTimeDuration-from-time(A, B) -> xs:time
        (Atomic::Time(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::Time(a)) => Ok(Atomic::Time(
            op_subtract_day_time_duration_from_time(a, *b)?,
        )),
        // op:subtract_dateTimes(A, B) -> xs:dayTimeDuration
        (Atomic::DateTime(a), Atomic::DateTime(b)) => {
            Ok(Atomic::DayTimeDuration(op_subtract_date_times(a, b)?))
        }
        (Atomic::DateTimeStamp(a), Atomic::DateTimeStamp(b)) => Ok(Atomic::DayTimeDuration(
            op_subtract_date_time_stamps(*a.as_ref(), *b.as_ref())?,
        )),
        (Atomic::DateTimeStamp(a), Atomic::DateTime(b)) => Ok(Atomic::DayTimeDuration(
            op_subtract_date_time_from_date_time_stamp(a, b)?,
        )),
        (Atomic::DateTime(a), Atomic::DateTimeStamp(b)) => Ok(Atomic::DayTimeDuration(
            op_subtract_date_time_stamp_from_date_time(a, b)?,
        )),
        // op:subtract-yearMonthDuration-from-dateTime(A, B) -> xs:dateTime
        (Atomic::DateTime(a), Atomic::YearMonthDuration(b))
        | (Atomic::YearMonthDuration(b), Atomic::DateTime(a)) => Ok(Atomic::DateTime(
            op_subtract_year_month_duration_from_date_time(a, b)?,
        )),
        // op:subtract-yearMonthDuration-from-dateTimeStamp(A, B) -> xs:dateTimeStamp
        (Atomic::DateTimeStamp(a), Atomic::YearMonthDuration(b))
        | (Atomic::YearMonthDuration(b), Atomic::DateTimeStamp(a)) => Ok(Atomic::DateTimeStamp(
            op_subtract_year_month_duration_from_date_time_stamp(a, b)?,
        )),
        // op:subtract-dayTimeDuration-from-dateTime(A, B) -> xs:dateTime
        (Atomic::DateTime(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::DateTime(a)) => Ok(Atomic::DateTime(
            op_subtract_day_time_duration_from_date_time(a, *b)?,
        )),
        // op:subtract-dayTimeDuration-from-dateTimeStamp(A, B) -> xs:dateTimeStamp
        (Atomic::DateTimeStamp(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::DateTimeStamp(a)) => Ok(Atomic::DateTimeStamp(
            op_subtract_day_time_duration_from_date_time_stamp(a, *b)?,
        )),
        // op:subtract-year-monthDurations(A, B) -> xs:yearMonthDuration
        (Atomic::YearMonthDuration(a), Atomic::YearMonthDuration(b)) => Ok(
            Atomic::YearMonthDuration(op_subtract_year_month_durations(a, b)?),
        ),
        // op:subtract-dayTimeDurations(A, B) -> xs:dayTimeDuration
        (Atomic::DayTimeDuration(a), Atomic::DayTimeDuration(b)) => Ok(Atomic::DayTimeDuration(
            op_subtract_day_time_durations(a, b)?,
        )),
        _ => Err(error::Error::Type),
    }
}

fn op_substract_decimal(a: Rc<Decimal>, b: Rc<Decimal>) -> error::Result<Rc<Decimal>> {
    Ok(Rc::new(
        a.as_ref()
            .checked_sub(*b.as_ref())
            .ok_or(error::Error::Overflow)?,
    ))
}

fn op_substract_integer(a: Rc<IBig>, b: Rc<IBig>) -> Rc<IBig> {
    Rc::new(a.as_ref() - b.as_ref())
}

fn op_subtract_year_month_duration_from_date(
    a: Rc<NaiveDateWithOffset>,
    b: YearMonthDuration,
) -> error::Result<Rc<NaiveDateWithOffset>> {
    let a = a.as_ref();
    let date = a.date;
    let new_date = if b.months >= 0 {
        date.checked_sub_months(chrono::Months::new(b.months as u32))
            .ok_or(error::Error::Overflow)
    } else {
        date.checked_add_months(chrono::Months::new(b.months.unsigned_abs() as u32))
            .ok_or(error::Error::Overflow)
    }?;

    Ok(Rc::new(NaiveDateWithOffset::new(new_date, a.offset)))
}

fn op_subtract_day_time_duration_from_date(
    a: Rc<NaiveDateWithOffset>,
    b: chrono::Duration,
) -> error::Result<Rc<NaiveDateWithOffset>> {
    let new_date = a
        .as_ref()
        .date
        .checked_sub_signed(b)
        .ok_or(error::Error::Overflow)?;
    Ok(Rc::new(NaiveDateWithOffset::new(
        new_date,
        a.as_ref().offset,
    )))
}

fn op_subtract_times(
    a: Rc<NaiveTimeWithOffset>,
    b: Rc<NaiveTimeWithOffset>,
) -> error::Result<Rc<chrono::Duration>> {
    let a = a.to_date_time_stamp();
    let b = b.to_date_time_stamp();
    op_subtract_date_time_stamps(a, b)
}

fn op_subtract_day_time_duration_from_time(
    a: Rc<NaiveTimeWithOffset>,
    b: chrono::Duration,
) -> error::Result<Rc<NaiveTimeWithOffset>> {
    // this never fails, but wraps around
    let new_time = a.as_ref().time - b;
    Ok(Rc::new(NaiveTimeWithOffset::new(
        new_time,
        a.as_ref().offset,
    )))
}

fn op_subtract_date_times(
    a: Rc<NaiveDateTimeWithOffset>,
    b: Rc<NaiveDateTimeWithOffset>,
) -> error::Result<Rc<chrono::Duration>> {
    let a = a.to_date_time_stamp();
    let b = b.to_date_time_stamp();
    op_subtract_date_time_stamps(a, b)
}

fn op_subtract_date_time_stamps(
    a: chrono::DateTime<chrono::FixedOffset>,
    b: chrono::DateTime<chrono::FixedOffset>,
) -> error::Result<Rc<chrono::Duration>> {
    Ok(Rc::new(a - b))
}

fn op_subtract_date_time_stamp_from_date_time(
    a: Rc<NaiveDateTimeWithOffset>,
    b: Rc<chrono::DateTime<chrono::FixedOffset>>,
) -> error::Result<Rc<chrono::Duration>> {
    let a = a.to_date_time_stamp();
    op_subtract_date_time_stamps(a, *b.as_ref())
}

fn op_subtract_date_time_from_date_time_stamp(
    a: Rc<chrono::DateTime<chrono::FixedOffset>>,
    b: Rc<NaiveDateTimeWithOffset>,
) -> error::Result<Rc<chrono::Duration>> {
    let b = b.to_date_time_stamp();
    op_subtract_date_time_stamps(*a.as_ref(), b)
}

fn op_subtract_year_month_duration_from_date_time(
    a: Rc<NaiveDateTimeWithOffset>,
    b: YearMonthDuration,
) -> error::Result<Rc<NaiveDateTimeWithOffset>> {
    let a = a.as_ref();
    let date_time = a.date_time;
    let new_date_time = if b.months >= 0 {
        date_time
            .checked_sub_months(chrono::Months::new(b.months as u32))
            .ok_or(error::Error::Overflow)
    } else {
        date_time
            .checked_add_months(chrono::Months::new(b.months.unsigned_abs() as u32))
            .ok_or(error::Error::Overflow)
    }?;

    Ok(Rc::new(NaiveDateTimeWithOffset::new(
        new_date_time,
        a.offset,
    )))
}

fn op_subtract_year_month_duration_from_date_time_stamp(
    a: Rc<chrono::DateTime<chrono::FixedOffset>>,
    b: YearMonthDuration,
) -> error::Result<Rc<chrono::DateTime<chrono::FixedOffset>>> {
    let a = a.as_ref();
    let date_time = *a;
    let new_date_time = if b.months >= 0 {
        date_time
            .checked_sub_months(chrono::Months::new(b.months as u32))
            .ok_or(error::Error::Overflow)
    } else {
        date_time
            .checked_add_months(chrono::Months::new(b.months.unsigned_abs() as u32))
            .ok_or(error::Error::Overflow)
    }?;

    Ok(Rc::new(new_date_time))
}

fn op_subtract_day_time_duration_from_date_time(
    a: Rc<NaiveDateTimeWithOffset>,
    b: chrono::Duration,
) -> error::Result<Rc<NaiveDateTimeWithOffset>> {
    let new_date_time = a
        .as_ref()
        .date_time
        .checked_sub_signed(b)
        .ok_or(error::Error::Overflow)?;
    Ok(Rc::new(NaiveDateTimeWithOffset::new(
        new_date_time,
        a.as_ref().offset,
    )))
}

fn op_subtract_day_time_duration_from_date_time_stamp(
    a: Rc<chrono::DateTime<chrono::FixedOffset>>,
    b: chrono::Duration,
) -> error::Result<Rc<chrono::DateTime<chrono::FixedOffset>>> {
    let new_date_time = (*a.as_ref())
        .checked_sub_signed(b)
        .ok_or(error::Error::Overflow)?;
    Ok(Rc::new(new_date_time))
}

fn op_subtract_year_month_durations(
    a: YearMonthDuration,
    b: YearMonthDuration,
) -> error::Result<YearMonthDuration> {
    let new_months = a
        .months
        .checked_sub(b.months)
        .ok_or(error::Error::Overflow)?;
    Ok(YearMonthDuration { months: new_months })
}

fn op_subtract_day_time_durations(
    a: Rc<chrono::Duration>,
    b: Rc<chrono::Duration>,
) -> error::Result<Rc<chrono::Duration>> {
    let new_duration = (*a.as_ref())
        .checked_sub(b.as_ref())
        .ok_or(error::Error::Overflow)?;

    Ok(Rc::new(new_duration))
}
