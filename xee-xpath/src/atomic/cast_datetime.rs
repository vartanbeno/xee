use std::cmp::Ordering;

use chrono::Offset;
use chrono::TimeZone;
use chrono::Timelike;
use chumsky::prelude::*;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

use super::cast::whitespace_collapse;

pub(crate) type BoxedParser<'a, 'b, T> = Boxed<'a, 'b, &'a str, T, extra::Default>;

impl atomic::Atomic {
    pub(crate) fn canonical_duration(months: i64, duration: chrono::Duration) -> String {
        // https://www.w3.org/TR/2012/REC-xmlschema11-2-20120405/datatypes.html#f-durationCanMap
        let mut s = String::new();
        if months < 0 || duration.num_milliseconds() < 0 {
            s.push('-');
        }
        s.push('P');
        if months != 0 && duration.num_milliseconds() != 0 {
            Self::push_canonical_year_month_duration_fragment(&mut s, months);
            Self::push_canonical_day_time_duration_fragment(&mut s, duration);
        } else if months != 0 {
            Self::push_canonical_year_month_duration_fragment(&mut s, months);
        } else {
            Self::push_canonical_day_time_duration_fragment(&mut s, duration);
        }
        s
    }

    pub(crate) fn canonical_year_month_duration(months: i64) -> String {
        let mut s = String::new();
        if months < 0 {
            s.push('-');
        }
        s.push('P');
        Self::push_canonical_year_month_duration_fragment(&mut s, months);
        s
    }

    fn push_canonical_year_month_duration_fragment(s: &mut String, months: i64) {
        // https://www.w3.org/TR/2012/REC-xmlschema11-2-20120405/datatypes.html#f-duYMCan
        let months = months.abs();
        let years = months / 12;
        let months = months % 12;
        if years != 0 && months != 0 {
            s.push_str(&format!("{}Y", years));
            s.push_str(&format!("{}M", months));
        } else if years != 0 {
            s.push_str(&format!("{}Y", years));
        } else {
            s.push_str(&format!("{}M", months));
        }
    }

    pub(crate) fn canonical_day_time_duration(duration: chrono::Duration) -> String {
        let mut s = String::new();
        if duration.num_milliseconds() < 0 {
            s.push('-');
        }
        s.push('P');
        Self::push_canonical_day_time_duration_fragment(&mut s, duration);
        s
    }

    fn push_canonical_day_time_duration_fragment(v: &mut String, duration: chrono::Duration) {
        // https://www.w3.org/TR/2012/REC-xmlschema11-2-20120405/datatypes.html#f-duDTCan
        let ss = duration.num_milliseconds().abs();
        let ss = (ss as f64) / 1000.0;
        if ss.is_zero() {
            v.push_str("T0S");
            return;
        }
        let d = (ss / 86400.0) as u64;
        let h = ((ss % 86400.0) / 3600.0) as u64;
        let m = ((ss % 3600.0) / 60.0) as u16;
        let s: Decimal = (ss % 60.0).try_into().unwrap_or(Decimal::from(0));

        if d != 0 {
            v.push_str(&format!("{}D", d));
        }
        if h != 0 || m != 0 || !s.is_zero() {
            v.push('T');
        }
        if h != 0 {
            v.push_str(&format!("{}H", h));
        }
        if m != 0 {
            v.push_str(&format!("{}M", m));
        }
        if s != Decimal::from(0) {
            v.push_str(&format!("{}S", s));
        }
    }

    pub(crate) fn canonical_date_time(
        date_time: chrono::NaiveDateTime,
        offset: Option<chrono::FixedOffset>,
    ) -> String {
        let mut s = String::new();
        s.push_str(&date_time.format("%Y-%m-%dT%H:%M:%S").to_string());
        let millis = date_time.timestamp_subsec_millis();
        if !millis.is_zero() {
            s.push_str(&format!(".{:03}", millis));
        }
        if let Some(offset) = offset {
            Self::push_canonical_time_zone_offset(&mut s, &offset);
        }
        s
    }

    pub(crate) fn canonical_date_time_stamp(
        date_time: chrono::DateTime<chrono::FixedOffset>,
    ) -> String {
        let mut s = String::new();
        s.push_str(&date_time.format("%Y-%m-%dT%H:%M:%S").to_string());
        let millis = date_time.timestamp_subsec_millis();
        if !millis.is_zero() {
            s.push_str(&format!(".{:03}", millis));
        }
        let offset = date_time.offset();
        Self::push_canonical_time_zone_offset(&mut s, offset);
        s
    }

    pub(crate) fn canonical_time(
        time: chrono::NaiveTime,
        offset: Option<chrono::FixedOffset>,
    ) -> String {
        let mut s = String::new();
        s.push_str(&time.format("%H:%M:%S").to_string());
        let millis = time.nanosecond() / 1_000_000;
        if !millis.is_zero() {
            s.push_str(&format!(".{:03}", millis));
        }
        if let Some(offset) = offset {
            Self::push_canonical_time_zone_offset(&mut s, &offset);
        }
        s
    }

    pub(crate) fn canonical_date(
        date: chrono::NaiveDate,
        offset: Option<chrono::FixedOffset>,
    ) -> String {
        let mut s = String::new();
        s.push_str(&date.format("%Y-%m-%d").to_string());
        if let Some(offset) = offset {
            Self::push_canonical_time_zone_offset(&mut s, &offset);
        }
        s
    }

    fn push_canonical_time_zone_offset(s: &mut String, offset: &chrono::FixedOffset) {
        let seconds = offset.local_minus_utc();
        if seconds == 0 {
            s.push('Z');
            return;
        }
        let is_negative = seconds < 0;
        let seconds = seconds.abs();
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let seconds = seconds % 60;
        if is_negative {
            s.push('-');
        } else {
            s.push('+');
        }
        s.push_str(&format!("{:02}:{:02}:{02}", hours, minutes, seconds));
    }

    // https://www.w3.org/TR/xpath-functions-31/#casting-to-durations

    pub(crate) fn cast_to_duration(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => Self::parse_duration(&s),
            atomic::Atomic::Duration(_, _) => Ok(self.clone()),
            atomic::Atomic::YearMonthDuration(months) => Ok(atomic::Atomic::Duration(
                months,
                chrono::Duration::seconds(0),
            )),
            atomic::Atomic::DayTimeDuration(duration) => Ok(atomic::Atomic::Duration(0, duration)),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_year_month_duration(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => {
                Self::parse_year_month_duration(&s)
            }
            atomic::Atomic::Duration(months, _) => Ok(atomic::Atomic::YearMonthDuration(months)),
            atomic::Atomic::YearMonthDuration(_) => Ok(self.clone()),
            atomic::Atomic::DayTimeDuration(_) => Ok(atomic::Atomic::YearMonthDuration(0)),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_day_time_duration(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => {
                Self::parse_day_time_duration(&s)
            }
            atomic::Atomic::Duration(_, duration) => Ok(atomic::Atomic::DayTimeDuration(duration)),
            atomic::Atomic::YearMonthDuration(_) => Ok(atomic::Atomic::DayTimeDuration(
                chrono::Duration::seconds(0),
            )),
            atomic::Atomic::DayTimeDuration(_) => Ok(self.clone()),
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_date_time(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => Self::parse_date_time(&s),
            atomic::Atomic::DateTime(_, _) => Ok(self.clone()),
            // TODO
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_date_time_stamp(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => {
                Self::parse_date_time_stamp(&s)
            }
            atomic::Atomic::DateTimeStamp(_) => Ok(self.clone()),
            // TODO
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_time(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => Self::parse_time(&s),
            atomic::Atomic::Time(_, _) => Ok(self.clone()),
            // TODO
            _ => Err(error::Error::Type),
        }
    }

    pub(crate) fn cast_to_date(self) -> error::Result<atomic::Atomic> {
        match self {
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => Self::parse_date(&s),
            atomic::Atomic::Date(_, _) => Ok(self.clone()),
            // TODO
            _ => Err(error::Error::Type),
        }
    }

    fn parse_duration(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = duration_parser();
        match parser.parse(&s).into_result() {
            Ok((months, duration)) => Ok(atomic::Atomic::Duration(months, duration)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }
    fn parse_year_month_duration(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = year_month_duration_parser();
        match parser.parse(&s).into_result() {
            Ok(months) => Ok(atomic::Atomic::YearMonthDuration(months)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }

    fn parse_day_time_duration(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = day_time_duration_parser();
        match parser.parse(&s).into_result() {
            Ok(duration) => Ok(atomic::Atomic::DayTimeDuration(duration)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }

    fn parse_date_time(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = date_time_parser();
        match parser.parse(&s).into_result() {
            Ok((date_time, tz)) => Ok(atomic::Atomic::DateTime(date_time, tz)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }

    fn parse_date_time_stamp(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = date_time_stamp_parser();
        match parser.parse(&s).into_result() {
            Ok(date_time) => Ok(atomic::Atomic::DateTimeStamp(date_time)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }

    fn parse_time(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = time_parser();
        match parser.parse(&s).into_result() {
            Ok((time, tz)) => Ok(atomic::Atomic::Time(time, tz)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }

    fn parse_date(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let s = whitespace_collapse(s);
        let parser = date_parser();
        match parser.parse(&s).into_result() {
            Ok((date, tz)) => Ok(atomic::Atomic::Date(date, tz)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }
}

fn digit_parser<'a>() -> impl Parser<'a, &'a str, char> {
    any::<&str, extra::Default>().filter(|c: &char| c.is_ascii_digit())
}

fn digits_parser<'a>() -> impl Parser<'a, &'a str, String> {
    let digit = digit_parser();
    digit.repeated().at_least(1).collect::<String>()
}

fn number_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    digits_parser().map(|s| s.parse().unwrap())
}

fn sign_parser<'a>() -> impl Parser<'a, &'a str, bool> {
    just('-').or_not().map(|sign| sign.is_some())
}

fn second_parser<'a>() -> impl Parser<'a, &'a str, (u32, u32)> {
    let digits = digits_parser().boxed();
    let seconds_with_fraction = digits
        .clone()
        .then_ignore(just('.'))
        .then(digits.clone())
        .map(|(a, b)| {
            // ignore anything below milliseconds
            let b = if b.len() > 3 { &b[..3] } else { &b };
            let l = b.len();

            let a = a.parse::<u32>().unwrap();
            let b = b.parse::<u32>().unwrap();
            (a, b * 10u32.pow(3 - l as u32))
        });
    let seconds_without_fraction = digits.map(|s| (s.parse::<u32>().unwrap(), 0));
    seconds_with_fraction.or(seconds_without_fraction)
}

fn year_month_fragment_parser<'a>() -> impl Parser<'a, &'a str, i64> {
    let number = number_parser().boxed();
    let year_y = number.clone().then_ignore(just('Y')).boxed();
    let month_m = number.then_ignore(just('M')).boxed();
    (year_y
        .clone()
        .then(month_m.clone())
        .map(|(years, months)| (years, months)))
    .or(year_y.map(|years| (years, 0)))
    .or(month_m.map(|months| (0, months)))
    .map(|(years, months)| years as i64 * 12 + months as i64)
}

fn year_month_duration_parser<'a>() -> impl Parser<'a, &'a str, i64> {
    let year_month = year_month_fragment_parser().boxed();
    let sign = sign_parser();
    sign.then_ignore(just('P'))
        .then(year_month.clone())
        .then_ignore(end())
        .map(|(sign, months)| if sign { -months } else { months })
}

fn day_time_fragment_parser<'a>() -> impl Parser<'a, &'a str, chrono::Duration> {
    let number = number_parser().boxed();
    let day_d = number.clone().then_ignore(just('D')).boxed();
    let hour_h = number.clone().then_ignore(just('H')).boxed();
    let minute_m = number.clone().then_ignore(just('M')).boxed();
    let second_s = second_parser().then_ignore(just('S')).boxed();

    let time = just('T')
        .ignore_then(hour_h.or_not())
        .then(minute_m.or_not())
        .then(second_s.or_not())
        .try_map(|((hours, minutes), s_ms), _| {
            if hours.is_none() && minutes.is_none() && s_ms.is_none() {
                return Err(EmptyErr::default());
            }
            let hours = hours.unwrap_or(0);
            let minutes = minutes.unwrap_or(0);
            let s_ms = s_ms.unwrap_or((0, 0));
            let (seconds, milliseconds) = s_ms;
            Ok(chrono::Duration::hours(hours as i64)
                + chrono::Duration::minutes(minutes as i64)
                + chrono::Duration::seconds(seconds as i64)
                + chrono::Duration::milliseconds(milliseconds as i64))
        })
        .boxed();

    let days = day_d.map(|days| chrono::Duration::days(days as i64));

    let days_then_time = days
        .then(time.clone().or_not())
        .map(|(days_duration, time_duration)| {
            let time_duration = time_duration.unwrap_or(chrono::Duration::seconds(0));
            days_duration + time_duration
        });
    days_then_time.or(time)
}

fn day_time_duration_parser<'a>() -> impl Parser<'a, &'a str, chrono::Duration> {
    let day_time = day_time_fragment_parser().boxed();
    let sign = sign_parser();
    sign.then_ignore(just('P'))
        .then(day_time.clone())
        .then_ignore(end())
        .map(|(sign, duration)| if sign { -duration } else { duration })
}

fn duration_parser<'a>() -> impl Parser<'a, &'a str, (i64, chrono::Duration)> {
    let year_month = year_month_fragment_parser().boxed();
    let day_time = day_time_fragment_parser().boxed();
    let sign = sign_parser();
    sign.then_ignore(just('P'))
        .then(year_month.clone().or_not())
        .then(day_time.clone().or_not())
        .then_ignore(end())
        .try_map(|((sign, months), duration), _| {
            if months.is_none() && duration.is_none() {
                return Err(EmptyErr::default());
            }
            let months = months.unwrap_or(0);
            let duration = duration.unwrap_or(chrono::Duration::seconds(0));
            if sign {
                Ok((-months, -duration))
            } else {
                Ok((months, duration))
            }
        })
}

fn year_parser<'a>() -> impl Parser<'a, &'a str, i32> {
    let digits = digits_parser();
    let sign = sign_parser();
    // the year may have 0 prefixes, unless it's larger than 4, in
    // which case we don't allow any prefixes
    sign.then(digits.boxed()).try_map(|(sign, digits), _| {
        let year = match digits.len().cmp(&4) {
            Ordering::Greater => {
                // cannot have any 0 prefix
                if digits.starts_with('0') {
                    Err(EmptyErr::default())
                } else {
                    Ok(digits.parse().unwrap())
                }
            }
            Ordering::Equal => Ok(digits.parse().unwrap()),
            Ordering::Less => Err(EmptyErr::default()),
        };
        year.map(|year: i32| if sign { -year } else { year })
    })
}

fn two_digit_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    let digit = digit_parser().boxed();
    digit
        .clone()
        .then(digit)
        .map(|(a, b)| a.to_digit(10).unwrap() * 10 + b.to_digit(10).unwrap())
}

fn month_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    two_digit_parser().try_map(|month, _| {
        if month == 0 || month > 12 {
            Err(EmptyErr::default())
        } else {
            Ok(month)
        }
    })
}

fn day_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    two_digit_parser().try_map(|day, _| {
        if day == 0 || day > 31 {
            Err(EmptyErr::default())
        } else {
            Ok(day)
        }
    })
}

fn date_fragment_parser<'a>() -> impl Parser<'a, &'a str, chrono::NaiveDate> {
    let year = year_parser().boxed();
    let month = month_parser().boxed();
    let day = day_parser().boxed();
    year.then_ignore(just('-'))
        .then(month)
        .then_ignore(just('-'))
        .then(day)
        .try_map(|((year, month), day), _| {
            chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(EmptyErr::default())
        })
}

fn date_parser<'a>() -> impl Parser<'a, &'a str, (chrono::NaiveDate, Option<chrono::FixedOffset>)> {
    let date = date_fragment_parser().boxed();
    let tz = tz_parser().boxed();
    date.then(tz.or_not()).then_ignore(end())
}

fn hour_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    two_digit_parser().try_map(|hour, _| {
        if hour > 24 {
            Err(EmptyErr::default())
        } else {
            Ok(hour)
        }
    })
}

fn minute_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    two_digit_parser().try_map(|minute, _| {
        if minute > 59 {
            Err(EmptyErr::default())
        } else {
            Ok(minute)
        }
    })
}

fn time_fragment_parser<'a>() -> impl Parser<'a, &'a str, chrono::NaiveTime> {
    let hour = hour_parser().boxed();
    let minute = minute_parser().boxed();
    let second = second_parser().boxed();
    hour.then_ignore(just(':'))
        .then(minute)
        .then_ignore(just(':'))
        .then(second)
        .try_map(|((hour, minute), (second, millisecond)), _| {
            chrono::NaiveTime::from_hms_milli_opt(hour, minute, second, millisecond)
                .ok_or(EmptyErr::default())
        })
}

fn time_parser<'a>() -> impl Parser<'a, &'a str, (chrono::NaiveTime, Option<chrono::FixedOffset>)> {
    let time = time_fragment_parser().boxed();
    let tz = tz_parser().boxed();
    time.then(tz.or_not()).then_ignore(end())
}

fn date_time_fragment_parser<'a>() -> impl Parser<'a, &'a str, chrono::NaiveDateTime> {
    let date = date_fragment_parser().boxed();
    let time = time_fragment_parser().boxed();
    date.then_ignore(just('T'))
        .then(time)
        .map(|(date, time)| date.and_time(time))
}

fn date_time_parser<'a>(
) -> impl Parser<'a, &'a str, (chrono::NaiveDateTime, Option<chrono::FixedOffset>)> {
    let date_time = date_time_fragment_parser().boxed();
    let tz = tz_parser().boxed();
    date_time.then(tz.or_not())
}

fn date_time_stamp_parser<'a>() -> impl Parser<'a, &'a str, chrono::DateTime<chrono::FixedOffset>> {
    let date_time = date_time_fragment_parser().boxed();
    let tz = tz_parser().boxed();
    date_time
        .then(tz)
        .map(|(date_time, tz)| tz.from_utc_datetime(&date_time))
}

fn offset_time_parser<'a>() -> impl Parser<'a, &'a str, i32> {
    let hour = hour_parser().boxed();
    let minute = minute_parser().boxed();
    hour.then_ignore(just(":"))
        .then(minute)
        .try_map(|(hour, minute), _| {
            if hour > 14 || hour == 14 && minute > 0 {
                Err(EmptyErr::default())
            } else {
                Ok(hour as i32 * 60 + minute as i32)
            }
        })
}

fn offset_parser<'a>() -> impl Parser<'a, &'a str, chrono::FixedOffset> {
    one_of("+-")
        .then(offset_time_parser())
        .map(|(sign, offset)| {
            // make it into seconds
            let offset = offset * 60;
            if sign == '+' {
                chrono::FixedOffset::east_opt(offset).unwrap()
            } else {
                chrono::FixedOffset::west_opt(offset).unwrap()
            }
        })
}

fn tz_parser<'a>() -> impl Parser<'a, &'a str, chrono::FixedOffset> {
    let offset = offset_parser();
    just('Z').to(chrono::offset::Utc.fix()).or(offset)
}

// pub(crate) struct DateTimeParsers<'input, 'parser: 'input> {
//     pub(crate) duration: BoxedParser<'input, 'parser, (i64, chrono::Duration)>,
//     pub(crate) year_month_duration: BoxedParser<'input, 'parser, i64>,
//     pub(crate) day_time_duration: BoxedParser<'input, 'parser, chrono::Duration>,
//     pub(crate) date_time:
//         BoxedParser<'input, 'parser, (chrono::NaiveDateTime, Option<chrono::FixedOffset>)>,
//     pub(crate) date_time_stamp: BoxedParser<'input, 'parser, chrono::DateTime<chrono::FixedOffset>>,
//     pub(crate) time: BoxedParser<'input, 'parser, (chrono::NaiveTime, Option<chrono::FixedOffset>)>,
//     pub(crate) date: BoxedParser<'input, 'parser, (chrono::NaiveDate, Option<chrono::FixedOffset>)>,
// }

// impl<'input, 'parser: 'input> DateTimeParsers<'input, 'parser> {
//     pub(crate) fn new() -> DateTimeParsers<'input, 'parser> {
//         let duration = duration_parser().boxed();
//         let year_month_duration = year_month_duration_parser().boxed();
//         let day_time_duration = day_time_duration_parser().boxed();
//         let date_time = date_time_parser().boxed();
//         let date_time_stamp = date_time_stamp_parser().boxed();
//         let time = time_parser().boxed();
//         let date = date_parser().boxed();
//         Self {
//             duration,
//             year_month_duration,
//             day_time_duration,
//             date_time,
//             date_time_stamp,
//             time,
//             date,
//         }
//     }

//     fn parse_duration<'s: 'input>(&'parser self, s: &'s str) -> error::Result<atomic::Atomic> {
//         let s = whitespace_collapse(s);
//         self.duration.parse(&s);
//         todo!();
//         // match self.duration.parse(&s).into_result() {
//         //     Ok((months, duration)) => Ok(atomic::Atomic::Duration(months, duration)),
//         //     Err(_) => Err(error::Error::FORG0001),
//         // }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_month_parser() {
        assert_eq!(year_month_fragment_parser().parse("1Y2M").unwrap(), 14);
    }

    #[test]
    fn test_year_month_parser_missing_year() {
        assert_eq!(year_month_fragment_parser().parse("2M").unwrap(), 2);
    }

    #[test]
    fn test_year_month_parser_missing_month() {
        assert_eq!(year_month_fragment_parser().parse("1Y").unwrap(), 12);
    }

    #[test]
    fn test_year_month_parser_zero_year() {
        assert_eq!(year_month_fragment_parser().parse("0Y2M").unwrap(), 2);
    }

    #[test]
    fn test_year_month_parser_leading_zero() {
        assert_eq!(year_month_fragment_parser().parse("01Y02M").unwrap(), 14);
    }

    #[test]
    fn test_year_month_duration_parser() {
        assert_eq!(year_month_duration_parser().parse("P1Y2M").unwrap(), 14);
    }

    #[test]
    fn test_year_month_duration_parser_negative() {
        assert_eq!(year_month_duration_parser().parse("-P1Y2M").unwrap(), -14);
    }

    #[test]
    fn test_year_month_duration_parser_extra_rejected() {
        assert!(year_month_duration_parser().parse("P1Yflurb").has_errors());
    }

    #[test]
    fn test_day_time_parser() {
        assert_eq!(
            day_time_fragment_parser().parse("1DT2H3M4S").unwrap(),
            chrono::Duration::days(1)
                + chrono::Duration::hours(2)
                + chrono::Duration::minutes(3)
                + chrono::Duration::seconds(4)
        );
    }

    #[test]
    fn test_day_time_parser_with_fraction_seconds() {
        assert_eq!(
            day_time_fragment_parser().parse("1DT2H3M4.5S").unwrap(),
            chrono::Duration::days(1)
                + chrono::Duration::hours(2)
                + chrono::Duration::minutes(3)
                + chrono::Duration::seconds(4)
                + chrono::Duration::milliseconds(500)
        );
    }

    #[test]
    fn test_day_time_parser_with_fraction_seconds_long() {
        assert_eq!(
            day_time_fragment_parser().parse("1DT2H3M4.5678S").unwrap(),
            chrono::Duration::days(1)
                + chrono::Duration::hours(2)
                + chrono::Duration::minutes(3)
                + chrono::Duration::seconds(4)
                + chrono::Duration::milliseconds(567)
        );
    }

    #[test]
    fn test_day_time_parser_just_days() {
        assert_eq!(
            day_time_fragment_parser().parse("1D").unwrap(),
            chrono::Duration::days(1)
        );
    }

    #[test]
    fn test_day_time_parser_just_time() {
        assert_eq!(
            day_time_fragment_parser().parse("T2H3M4S").unwrap(),
            chrono::Duration::hours(2)
                + chrono::Duration::minutes(3)
                + chrono::Duration::seconds(4)
        );
    }

    #[test]
    fn test_day_time_parser_just_seconds() {
        assert_eq!(
            day_time_fragment_parser().parse("T4S").unwrap(),
            chrono::Duration::seconds(4)
        );
    }

    #[test]
    fn test_day_time_parser_empty_fails() {
        assert!(day_time_fragment_parser().parse("").has_errors());
    }

    #[test]
    fn test_day_time_parser_just_t_fails() {
        assert!(day_time_fragment_parser().parse("T").has_errors());
    }

    #[test]
    fn test_duration_parser() {
        assert_eq!(
            duration_parser().parse("P1Y2M3DT4H5M6S").unwrap(),
            (
                14,
                chrono::Duration::days(3)
                    + chrono::Duration::hours(4)
                    + chrono::Duration::minutes(5)
                    + chrono::Duration::seconds(6)
            )
        );
    }

    #[test]
    fn test_duration_parser_just_months() {
        assert_eq!(
            duration_parser().parse("P1Y2M").unwrap(),
            (14, chrono::Duration::seconds(0))
        );
    }

    #[test]
    fn test_duration_parser_just_days() {
        assert_eq!(
            duration_parser().parse("P1D").unwrap(),
            (0, chrono::Duration::days(1))
        );
    }

    #[test]
    fn test_duration_parser_nothing() {
        assert!(duration_parser().parse("P").has_errors());
    }

    #[test]
    fn test_date_parser_4_digit_year() {
        assert_eq!(
            date_parser().parse("2020-01-02").unwrap(),
            (chrono::NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(), None)
        );
    }

    #[test]
    fn test_date_parser_more_digits_year() {
        assert_eq!(
            date_parser().parse("20200-01-02").unwrap(),
            (chrono::NaiveDate::from_ymd_opt(20200, 1, 2).unwrap(), None)
        );
    }

    #[test]
    fn test_date_parser_year_with_zeros() {
        assert_eq!(
            date_parser().parse("0120-01-02").unwrap(),
            (chrono::NaiveDate::from_ymd_opt(120, 1, 2).unwrap(), None)
        );
    }

    #[test]
    fn test_date_parser_wrong_month() {
        assert!(date_parser().parse("2020-13-02").has_errors());
    }

    #[test]
    fn test_date_parser_wrong_day() {
        assert!(date_parser().parse("2020-01-32").has_errors());
    }

    #[test]
    fn test_date_parser_early_year_without_zeros_fails() {
        assert!(date_parser().parse("120-01-02").has_errors());
    }

    #[test]
    fn test_date_parser_long_year_leading_zeros_fails() {
        assert!(date_parser().parse("012020-01-02").has_errors());
    }

    #[test]
    fn test_date_parser_junk_fails() {
        assert!(date_parser().parse("2020-01-02flurb").has_errors());
    }

    #[test]
    fn test_date_parser_utc() {
        assert_eq!(
            date_parser().parse("2020-01-02Z").unwrap(),
            (
                chrono::NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                Some(chrono::offset::Utc.fix())
            )
        );
    }

    #[test]
    fn test_tz_parser_utc() {
        assert_eq!(tz_parser().parse("Z").unwrap(), chrono::offset::Utc.fix());
    }

    #[test]
    fn test_tz_parser_naive() {
        assert_eq!(tz_parser().or_not().parse("").unwrap(), None);
    }

    #[test]
    fn test_tz_parser_offset_east() {
        assert_eq!(
            tz_parser().parse("+01:00").unwrap(),
            chrono::FixedOffset::east_opt(3600).unwrap()
        );
    }

    #[test]
    fn test_tz_parser_offset_west() {
        assert_eq!(
            tz_parser().parse("-01:00").unwrap(),
            chrono::FixedOffset::west_opt(3600).unwrap()
        );
    }

    #[test]
    fn test_tz_parser_offset_too_big_fails() {
        assert!(tz_parser().parse("+15:00").has_errors());
    }

    #[test]
    fn test_tz_parser_offset_max_range() {
        assert_eq!(
            tz_parser().parse("+14:00").unwrap(),
            chrono::FixedOffset::east_opt(50400).unwrap()
        );
    }

    #[test]
    fn test_tz_parser_offset_too_big2_fails() {
        assert!(tz_parser().parse("+14:01").has_errors());
    }

    #[test]
    fn test_tz_parser_offset_wrong_minutes_fails() {
        assert!(tz_parser().parse("+01:60").has_errors());
    }

    #[test]
    fn test_time_parser() {
        assert_eq!(
            time_parser().parse("01:02:03.456").unwrap(),
            (
                chrono::NaiveTime::from_hms_milli_opt(1, 2, 3, 456).unwrap(),
                None
            )
        );
    }

    #[test]
    fn test_time_parser_no_ms() {
        assert_eq!(
            time_parser().parse("01:02:03").unwrap(),
            (
                chrono::NaiveTime::from_hms_milli_opt(1, 2, 3, 0).unwrap(),
                None
            )
        );
    }

    #[test]
    fn test_time_parser_utc() {
        assert_eq!(
            time_parser().parse("01:02:03.456Z").unwrap(),
            (
                chrono::NaiveTime::from_hms_milli_opt(1, 2, 3, 456).unwrap(),
                Some(chrono::offset::Utc.fix())
            )
        );
    }

    #[test]
    fn test_time_parser_fails() {
        assert!(time_parser().parse("25:00:00").has_errors());
    }

    #[test]
    fn test_time_parser_junk_fails() {
        assert!(time_parser().parse("01:02:03.456flurb").has_errors());
    }

    #[test]
    fn test_date_time_parser() {
        assert_eq!(
            date_time_parser().parse("2020-01-02T01:02:03.456").unwrap(),
            (
                chrono::NaiveDate::from_ymd_opt(2020, 1, 2)
                    .unwrap()
                    .and_hms_milli_opt(1, 2, 3, 456)
                    .unwrap(),
                None
            )
        );
    }

    #[test]
    fn test_date_time_parser_utc() {
        assert_eq!(
            date_time_parser()
                .parse("2020-01-02T01:02:03.456Z")
                .unwrap(),
            (
                chrono::NaiveDate::from_ymd_opt(2020, 1, 2)
                    .unwrap()
                    .and_hms_milli_opt(1, 2, 3, 456)
                    .unwrap(),
                Some(chrono::offset::Utc.fix())
            )
        );
    }

    #[test]
    fn test_date_time_parser_offset() {
        assert_eq!(
            date_time_parser()
                .parse("2020-01-02T01:02:03.456+01:00")
                .unwrap(),
            (
                chrono::NaiveDate::from_ymd_opt(2020, 1, 2)
                    .unwrap()
                    .and_hms_milli_opt(1, 2, 3, 456)
                    .unwrap(),
                Some(chrono::FixedOffset::east_opt(3600).unwrap())
            )
        );
    }

    #[test]
    fn test_date_time_parser_junk_fails() {
        assert!(date_time_parser()
            .parse("2020-01-02T01:02:03.456flurb")
            .has_errors());
    }
}
