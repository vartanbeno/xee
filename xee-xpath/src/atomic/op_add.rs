use std::rc::Rc;

use ibig::IBig;
use rust_decimal::Decimal;

use crate::atomic;
use crate::error;

use super::cast_numeric::cast_numeric;
use super::datetime::{
    NaiveDateTimeWithOffset, NaiveDateWithOffset, NaiveTimeWithOffset, YearMonthDuration,
};

pub(crate) fn op_add(a: atomic::Atomic, b: atomic::Atomic) -> error::Result<atomic::Atomic> {
    use atomic::Atomic;

    let (a, b) = cast_numeric(a, b)?;

    match (a, b) {
        (Atomic::Decimal(a), Atomic::Decimal(b)) => Ok(op_add_decimal(a, b)?),
        (Atomic::Integer(_, a), Atomic::Integer(_, b)) => Ok(op_add_integer(a, b)),
        (Atomic::Float(a), Atomic::Float(b)) => Ok((a + b).into()),
        (Atomic::Double(a), Atomic::Double(b)) => Ok((a + b).into()),
        // op:add-yearMonthDuration-to-date(A, B) -> xs:date
        (Atomic::Date(a), Atomic::YearMonthDuration(b))
        | (Atomic::YearMonthDuration(b), Atomic::Date(a)) => {
            Ok(op_add_year_month_duration_to_date(a, b)?)
        }
        // op:add-dayTimeDuration-to-date(A, B) -> xs:date
        (Atomic::Date(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::Date(a)) => {
            Ok(op_add_day_time_duration_to_date(a, *b)?)
        }
        // op:add-dayTimeDuration-to-time(A, B) -> xs:time
        (Atomic::Time(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::Time(a)) => {
            Ok(op_add_day_time_duration_to_time(a, *b)?)
        }
        // op:add-yearMonthDuration-to-dateTime(A, B) -> xs:dateTime
        (Atomic::DateTime(a), Atomic::YearMonthDuration(b))
        | (Atomic::YearMonthDuration(b), Atomic::DateTime(a)) => {
            Ok(op_add_year_month_duration_to_date_time(a, b)?)
        }
        // op:add-yearMonthDuration-to-dateTimeStamp(A, B) -> xs:dateTimeStamp
        (Atomic::DateTimeStamp(a), Atomic::YearMonthDuration(b))
        | (Atomic::YearMonthDuration(b), Atomic::DateTimeStamp(a)) => {
            Ok(op_add_year_month_duration_to_date_time_stamp(a, b)?)
        }
        // op:add-dayTimeDuration-to-dateTime(A, B) -> xs:dateTime
        (Atomic::DateTime(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::DateTime(a)) => {
            Ok(op_add_day_time_duration_to_date_time(a, *b)?)
        }
        // op:add-dayTimeDuration-to-dateTimeStamp(A, B) -> xs:dateTimeStamp
        (Atomic::DateTimeStamp(a), Atomic::DayTimeDuration(b))
        | (Atomic::DayTimeDuration(b), Atomic::DateTimeStamp(a)) => {
            Ok(op_add_day_time_duration_to_date_time_stamp(a, *b)?)
        }
        // op:add-year-monthDurations(A, B) -> xs:yearMonthDuration
        (Atomic::YearMonthDuration(a), Atomic::YearMonthDuration(b)) => {
            Ok(op_add_year_month_durations(a, b)?)
        }
        // op:add-dayTimeDurations(A, B) -> xs:dayTimeDuration
        (Atomic::DayTimeDuration(a), Atomic::DayTimeDuration(b)) => {
            Ok(op_add_day_time_durations(a, b)?)
        }
        _ => Err(error::Error::Type),
    }
}

fn op_add_decimal(a: Rc<Decimal>, b: Rc<Decimal>) -> error::Result<atomic::Atomic> {
    Ok(a.as_ref()
        .checked_add(*b.as_ref())
        .ok_or(error::Error::Overflow)?
        .into())
}

fn op_add_integer(a: Rc<IBig>, b: Rc<IBig>) -> atomic::Atomic {
    (a.as_ref() + b.as_ref()).into()
}

fn op_add_year_month_duration_to_date(
    a: Rc<NaiveDateWithOffset>,
    b: YearMonthDuration,
) -> error::Result<atomic::Atomic> {
    let a = a.as_ref();
    let date = a.date;
    let new_date = if b.months >= 0 {
        date.checked_add_months(chrono::Months::new(b.months as u32))
            .ok_or(error::Error::Overflow)
    } else {
        date.checked_sub_months(chrono::Months::new(b.months.unsigned_abs() as u32))
            .ok_or(error::Error::Overflow)
    }?;

    Ok(NaiveDateWithOffset::new(new_date, a.offset).into())
}

fn op_add_day_time_duration_to_date(
    a: Rc<NaiveDateWithOffset>,
    b: chrono::Duration,
) -> error::Result<atomic::Atomic> {
    let new_date = a
        .as_ref()
        .date
        .checked_add_signed(b)
        .ok_or(error::Error::Overflow)?;
    Ok(NaiveDateWithOffset::new(new_date, a.as_ref().offset).into())
}

fn op_add_day_time_duration_to_time(
    a: Rc<NaiveTimeWithOffset>,
    b: chrono::Duration,
) -> error::Result<atomic::Atomic> {
    // this never fails, but wraps around
    let new_time = a.as_ref().time + b;
    Ok(NaiveTimeWithOffset::new(new_time, a.as_ref().offset).into())
}

fn op_add_year_month_duration_to_date_time(
    a: Rc<NaiveDateTimeWithOffset>,
    b: YearMonthDuration,
) -> error::Result<atomic::Atomic> {
    let a = a.as_ref();
    let date_time = a.date_time;
    let new_date_time = if b.months >= 0 {
        date_time
            .checked_add_months(chrono::Months::new(b.months as u32))
            .ok_or(error::Error::Overflow)
    } else {
        date_time
            .checked_sub_months(chrono::Months::new(b.months.unsigned_abs() as u32))
            .ok_or(error::Error::Overflow)
    }?;

    Ok(NaiveDateTimeWithOffset::new(new_date_time, a.offset).into())
}

fn op_add_year_month_duration_to_date_time_stamp(
    a: Rc<chrono::DateTime<chrono::FixedOffset>>,
    b: YearMonthDuration,
) -> error::Result<atomic::Atomic> {
    let a = a.as_ref();
    let date_time = *a;
    let new_date_time = if b.months >= 0 {
        date_time
            .checked_add_months(chrono::Months::new(b.months as u32))
            .ok_or(error::Error::Overflow)
    } else {
        date_time
            .checked_sub_months(chrono::Months::new(b.months.unsigned_abs() as u32))
            .ok_or(error::Error::Overflow)
    }?;

    Ok(new_date_time.into())
}

fn op_add_day_time_duration_to_date_time(
    a: Rc<NaiveDateTimeWithOffset>,
    b: chrono::Duration,
) -> error::Result<atomic::Atomic> {
    let new_date_time = a
        .as_ref()
        .date_time
        .checked_add_signed(b)
        .ok_or(error::Error::Overflow)?;
    Ok(NaiveDateTimeWithOffset::new(new_date_time, a.as_ref().offset).into())
}

fn op_add_day_time_duration_to_date_time_stamp(
    a: Rc<chrono::DateTime<chrono::FixedOffset>>,
    b: chrono::Duration,
) -> error::Result<atomic::Atomic> {
    let new_date_time = (*a.as_ref())
        .checked_add_signed(b)
        .ok_or(error::Error::Overflow)?;
    Ok(new_date_time.into())
}

fn op_add_year_month_durations(
    a: YearMonthDuration,
    b: YearMonthDuration,
) -> error::Result<atomic::Atomic> {
    let new_months = a
        .months
        .checked_add(b.months)
        .ok_or(error::Error::Overflow)?;
    Ok(YearMonthDuration { months: new_months }.into())
}

fn op_add_day_time_durations(
    a: Rc<chrono::Duration>,
    b: Rc<chrono::Duration>,
) -> error::Result<atomic::Atomic> {
    let new_duration = (*a.as_ref())
        .checked_add(b.as_ref())
        .ok_or(error::Error::Overflow)?;

    Ok(new_duration.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;

    #[test]
    fn test_decimal_add() {
        assert_eq!(
            op_add(dec!(1.5).into(), dec!(3.7).into()),
            Ok(dec!(5.2).into())
        );
    }

    #[test]
    fn test_add_decimals_overflow() {
        let a = Decimal::MAX.into();
        let b = dec!(2.7).into();
        let result = op_add(a, b);
        assert_eq!(result, Err(error::Error::Overflow));
    }

    #[test]
    fn test_integer_add() {
        assert_eq!(op_add(1i64.into(), 2i64.into()), Ok(3i64.into()));
    }

    #[test]
    fn test_float_add() {
        assert_eq!(op_add(1.5f32.into(), 2.5f32.into()), Ok(4.0f32.into()));
    }

    #[test]
    fn test_double_add() {
        assert_eq!(op_add(1.5f64.into(), 2.5f64.into()), Ok(4.0f64.into()));
    }

    #[test]
    fn test_decimal_float_add() {
        assert_eq!(op_add(dec!(1.5).into(), 2.5f32.into()), Ok(4.0f32.into()));
    }

    #[test]
    fn test_add_integer_decimal() {
        let a = 1i64.into();
        let b = dec!(2.7).into();
        let result = op_add(a, b).unwrap();
        assert_eq!(result, dec!(3.7).into());
    }

    #[test]
    fn test_add_double_decimal() {
        let a = 1.5f64.into();
        let b = dec!(2.7).into();
        let result = op_add(a, b).unwrap();
        assert_eq!(result, dec!(4.2).into());
    }
}
