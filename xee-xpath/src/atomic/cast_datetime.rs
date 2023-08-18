use std::cmp::Ordering;

use chumsky::prelude::*;
use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

use super::cast::whitespace_collapse;

pub(crate) type BoxedParser<'a, T> = Boxed<'a, 'a, &'a str, T, extra::Default>;

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

fn milliseconds_parser<'a>() -> impl Parser<'a, &'a str, u32> {
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
            a * 1000 + b * 10u32.pow(3 - l as u32)
        });
    let seconds_without_fraction = digits.map(|s| s.parse::<u32>().unwrap() * 1000);
    seconds_with_fraction.or(seconds_without_fraction)
}

fn year_month_parser<'a>() -> impl Parser<'a, &'a str, i64> {
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
    let year_month = year_month_parser().boxed();
    let sign = sign_parser();
    sign.then_ignore(just('P'))
        .then(year_month.clone())
        .then_ignore(end())
        .map(|(sign, months)| if sign { -months } else { months })
}

fn day_time_parser<'a>() -> impl Parser<'a, &'a str, chrono::Duration> {
    let number = number_parser().boxed();
    let day_d = number.clone().then_ignore(just('D')).boxed();
    let hour_h = number.clone().then_ignore(just('H')).boxed();
    let minute_m = number.clone().then_ignore(just('M')).boxed();
    let second_s = milliseconds_parser().then_ignore(just('S')).boxed();

    let time = just('T')
        .ignore_then(hour_h.or_not())
        .then(minute_m.or_not())
        .then(second_s.or_not())
        .try_map(|((hours, minutes), milliseconds), _| {
            if hours.is_none() && minutes.is_none() && milliseconds.is_none() {
                return Err(EmptyErr::default());
            }
            let hours = hours.unwrap_or(0);
            let minutes = minutes.unwrap_or(0);
            let milliseconds = milliseconds.unwrap_or(0);
            Ok(chrono::Duration::hours(hours as i64)
                + chrono::Duration::minutes(minutes as i64)
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
    let day_time = day_time_parser().boxed();
    let sign = sign_parser();
    sign.then_ignore(just('P'))
        .then(day_time.clone())
        .then_ignore(end())
        .map(|(sign, duration)| if sign { -duration } else { duration })
}

fn duration_parser<'a>() -> impl Parser<'a, &'a str, (i64, chrono::Duration)> {
    let year_month = year_month_parser().boxed();
    let day_time = day_time_parser().boxed();
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

fn month_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    let digit = digit_parser().boxed();
    digit.clone().then(digit).try_map(|(a, b), _| {
        let month = a.to_digit(10).unwrap() * 10 + b.to_digit(10).unwrap();
        if month == 0 || month > 12 {
            Err(EmptyErr::default())
        } else {
            Ok(month)
        }
    })
}

fn day_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    let digit = digit_parser().boxed();
    digit.clone().then(digit).try_map(|(a, b), _| {
        let day = a.to_digit(10).unwrap() * 10 + b.to_digit(10).unwrap();
        if day == 0 || day > 31 {
            Err(EmptyErr::default())
        } else {
            Ok(day)
        }
    })
}

fn date_parser<'a>() -> impl Parser<'a, &'a str, chrono::NaiveDate> {
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

// fn tz_parser<'a>() -> impl Parser<'a, &'a str, Option<chrono::FixedOffset>> {
//     just('-');
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_month_parser() {
        assert_eq!(year_month_parser().parse("1Y2M").unwrap(), 14);
    }

    #[test]
    fn test_year_month_parser_missing_year() {
        assert_eq!(year_month_parser().parse("2M").unwrap(), 2);
    }

    #[test]
    fn test_year_month_parser_missing_month() {
        assert_eq!(year_month_parser().parse("1Y").unwrap(), 12);
    }

    #[test]
    fn test_year_month_parser_zero_year() {
        assert_eq!(year_month_parser().parse("0Y2M").unwrap(), 2);
    }

    #[test]
    fn test_year_month_parser_leading_zero() {
        assert_eq!(year_month_parser().parse("01Y02M").unwrap(), 14);
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
            day_time_parser().parse("1DT2H3M4S").unwrap(),
            chrono::Duration::days(1)
                + chrono::Duration::hours(2)
                + chrono::Duration::minutes(3)
                + chrono::Duration::seconds(4)
        );
    }

    #[test]
    fn test_day_time_parser_with_fraction_seconds() {
        assert_eq!(
            day_time_parser().parse("1DT2H3M4.5S").unwrap(),
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
            day_time_parser().parse("1DT2H3M4.5678S").unwrap(),
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
            day_time_parser().parse("1D").unwrap(),
            chrono::Duration::days(1)
        );
    }

    #[test]
    fn test_day_time_parser_just_time() {
        assert_eq!(
            day_time_parser().parse("T2H3M4S").unwrap(),
            chrono::Duration::hours(2)
                + chrono::Duration::minutes(3)
                + chrono::Duration::seconds(4)
        );
    }

    #[test]
    fn test_day_time_parser_just_seconds() {
        assert_eq!(
            day_time_parser().parse("T4S").unwrap(),
            chrono::Duration::seconds(4)
        );
    }

    #[test]
    fn test_day_time_parser_empty_fails() {
        assert!(day_time_parser().parse("").has_errors());
    }

    #[test]
    fn test_day_time_parser_just_t_fails() {
        assert!(day_time_parser().parse("T").has_errors());
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
            chrono::NaiveDate::from_ymd_opt(2020, 1, 2).unwrap()
        );
    }

    #[test]
    fn test_date_parser_more_digits_year() {
        assert_eq!(
            date_parser().parse("20200-01-02").unwrap(),
            chrono::NaiveDate::from_ymd_opt(20200, 1, 2).unwrap()
        );
    }

    #[test]
    fn test_date_parser_year_with_zeros() {
        assert_eq!(
            date_parser().parse("0120-01-02").unwrap(),
            chrono::NaiveDate::from_ymd_opt(120, 1, 2).unwrap()
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
}
