// https://www.w3.org/TR/xpath-functions-31/#dates-times

use chrono::{Datelike, SubsecRound, Timelike};
use ibig::IBig;
use rust_decimal::Decimal;
use xee_xpath_macros::xpath_fn;

use crate::context::StaticFunctionDescription;
use crate::{
    error, wrap_xpath_fn, NaiveDateTimeWithOffset, NaiveDateWithOffset, NaiveTimeWithOffset,
};

#[xpath_fn("fn:dateTime($arg1 as xs:date?, $arg2 as xs:time?) as xs:dateTime?")]
fn date_time(
    arg1: Option<NaiveDateWithOffset>,
    arg2: Option<NaiveTimeWithOffset>,
) -> error::Result<Option<NaiveDateTimeWithOffset>> {
    match (arg1, arg2) {
        (Some(arg1), Some(arg2)) => {
            let offset = match (arg1.offset, arg2.offset) {
                (Some(arg1), Some(arg2)) => {
                    if arg1 == arg2 {
                        Some(arg1)
                    } else {
                        return Err(error::Error::FORG0008);
                    }
                }
                (Some(arg1), None) => Some(arg1),
                (None, Some(arg2)) => Some(arg2),
                (None, None) => None,
            };
            Ok(Some(NaiveDateTimeWithOffset::new(
                arg1.date.and_time(arg2.time),
                offset,
            )))
        }
        (Some(_), None) => Ok(None),
        (None, Some(_)) => Ok(None),
        (None, None) => Ok(None),
    }
}

#[xpath_fn("fn:year-from-dateTime($arg as xs:dateTime?) as xs:integer?")]
fn year_from_date_time(arg: Option<NaiveDateTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date_time.year().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:month-from-dateTime($arg as xs:dateTime?) as xs:integer?")]
fn month_from_date_time(arg: Option<NaiveDateTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date_time.month().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:day-from-dateTime($arg as xs:dateTime?) as xs:integer?")]
fn day_from_date_time(arg: Option<NaiveDateTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date_time.day().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:hours-from-dateTime($arg as xs:dateTime?) as xs:integer?")]
fn hours_from_date_time(arg: Option<NaiveDateTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date_time.hour().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:minutes-from-dateTime($arg as xs:dateTime?) as xs:integer?")]
fn minutes_from_date_time(arg: Option<NaiveDateTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date_time.minute().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:seconds-from-dateTime($arg as xs:dateTime?) as xs:decimal?")]
fn seconds_from_date_time(arg: Option<NaiveDateTimeWithOffset>) -> error::Result<Option<Decimal>> {
    match arg {
        Some(arg) => Ok(Some(seconds(arg.date_time))),
        None => Ok(None),
    }
}

#[xpath_fn("fn:timezone-from-dateTime($arg as xs:dateTime?) as xs:dayTimeDuration?")]
fn timezone_from_date_time(
    arg: Option<NaiveDateTimeWithOffset>,
) -> error::Result<Option<chrono::Duration>> {
    match arg {
        Some(arg) => Ok(duration(arg.offset)),
        None => Ok(None),
    }
}

#[xpath_fn("fn:year-from-date($arg as xs:date?) as xs:integer?")]
fn year_from_date(arg: Option<NaiveDateWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date.year().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:month-from-date($arg as xs:date?) as xs:integer?")]
fn month_from_date(arg: Option<NaiveDateWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date.month().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:day-from-date($arg as xs:date?) as xs:integer?")]
fn day_from_date(arg: Option<NaiveDateWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.date.day().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:timezone-from-date($arg as xs:date?) as xs:time?")]
fn timezone_from_date(arg: Option<NaiveDateWithOffset>) -> error::Result<Option<chrono::Duration>> {
    match arg {
        Some(arg) => Ok(duration(arg.offset)),
        None => Ok(None),
    }
}

#[xpath_fn("fn:hours-from-time($arg as xs:time?) as xs:integer?")]
fn hours_from_time(arg: Option<NaiveTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.time.hour().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:minutes-from-time($arg as xs:time?) as xs:integer?")]
fn minutes_from_time(arg: Option<NaiveTimeWithOffset>) -> error::Result<Option<IBig>> {
    match arg {
        Some(arg) => Ok(Some(arg.time.minute().into())),
        None => Ok(None),
    }
}

#[xpath_fn("fn:seconds-from-time($arg as xs:time?) as xs:decimal?")]
fn seconds_from_time(arg: Option<NaiveTimeWithOffset>) -> error::Result<Option<Decimal>> {
    match arg {
        Some(arg) => Ok(Some(seconds(arg.time))),
        None => Ok(None),
    }
}

#[xpath_fn("fn:timezone-from-time($arg as xs:time?) as xs:dayTimeDuration?")]
fn timezone_from_time(arg: Option<NaiveTimeWithOffset>) -> error::Result<Option<chrono::Duration>> {
    match arg {
        Some(arg) => Ok(duration(arg.offset)),
        None => Ok(None),
    }
}

fn seconds(time: impl Timelike + SubsecRound + Copy) -> Decimal {
    let nanoseconds: Decimal = time.round_subsecs(3).nanosecond().into();
    let seconds: Decimal = time.second().into();
    seconds + (nanoseconds / Decimal::from(1_000_000_000))
}

fn duration(offset: Option<chrono::FixedOffset>) -> Option<chrono::Duration> {
    offset.map(|offset| chrono::Duration::seconds(offset.local_minus_utc() as i64))
}

pub(crate) fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        wrap_xpath_fn!(date_time),
        wrap_xpath_fn!(year_from_date_time),
        wrap_xpath_fn!(month_from_date_time),
        wrap_xpath_fn!(day_from_date_time),
        wrap_xpath_fn!(hours_from_date_time),
        wrap_xpath_fn!(minutes_from_date_time),
        wrap_xpath_fn!(seconds_from_date_time),
        wrap_xpath_fn!(timezone_from_date_time),
        wrap_xpath_fn!(year_from_date),
        wrap_xpath_fn!(month_from_date),
        wrap_xpath_fn!(day_from_date),
        wrap_xpath_fn!(timezone_from_date),
        wrap_xpath_fn!(hours_from_time),
        wrap_xpath_fn!(minutes_from_time),
        wrap_xpath_fn!(seconds_from_time),
        wrap_xpath_fn!(timezone_from_time),
    ]
}
