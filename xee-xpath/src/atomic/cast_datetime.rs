use chumsky::prelude::*;
use rust_decimal::prelude::*;
use std::sync::OnceLock;

use crate::atomic;
use crate::error;

use super::cast::whitespace_collapse;

pub(crate) type BoxedParser<'a, T> = Boxed<'a, 'a, &'a str, T, Simple<'a, char>>;

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
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => todo!(),
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
            atomic::Atomic::Untyped(s) | atomic::Atomic::String(_, s) => todo!(),
            atomic::Atomic::Duration(_, duration) => Ok(atomic::Atomic::DayTimeDuration(duration)),
            atomic::Atomic::YearMonthDuration(_) => Ok(atomic::Atomic::DayTimeDuration(
                chrono::Duration::seconds(0),
            )),
            atomic::Atomic::DayTimeDuration(_) => Ok(self.clone()),
            _ => Err(error::Error::Type),
        }
    }

    fn parse_year_month_duration(s: &str) -> error::Result<atomic::Atomic> {
        // TODO: this has overhead I'd like to avoid
        // https://github.com/zesterer/chumsky/issues/501
        let parser = year_month_duration_parser();
        match parser.parse(s).into_result() {
            Ok(months) => Ok(atomic::Atomic::YearMonthDuration(months)),
            Err(_) => Err(error::Error::FORG0001),
        }
    }
}

fn number_parser<'a>() -> impl Parser<'a, &'a str, u32> {
    let number_leading_zero = just('0').repeated();
    let non_zero_number = number_leading_zero
        .ignore_then(text::int(10))
        .map(|s: &str| s.parse().unwrap());
    let zero_number = number_leading_zero.map(|_| 0);
    non_zero_number.or(zero_number)
}

fn year_month_parser<'a>() -> impl Parser<'a, &'a str, (u32, u32)> {
    let number = number_parser().boxed();
    let year_y = number.clone().then_ignore(just('Y')).boxed();
    let month_m = number.then_ignore(just('M')).boxed();
    (year_y
        .clone()
        .then(month_m.clone())
        .map(|(years, months)| (years, months)))
    .or(year_y.map(|years| (years, 0)))
    .or(month_m.map(|months| (0, months)))
}

fn year_month_duration_parser<'a>() -> impl Parser<'a, &'a str, i64> {
    let year_month = year_month_parser().boxed();
    let sign = just('-').or_not().map(|sign| sign.is_some());
    sign.then_ignore(just('P'))
        .then(year_month.clone())
        .then_ignore(end())
        .map(|(sign, (years, months))| {
            let total = years as i64 * 12 + months as i64;
            if sign {
                -total
            } else {
                total
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_month_parser() {
        assert_eq!(year_month_parser().parse("1Y2M").unwrap(), (1, 2));
    }

    #[test]
    fn test_year_month_parser_missing_year() {
        assert_eq!(year_month_parser().parse("2M").unwrap(), (0, 2));
    }

    #[test]
    fn test_year_month_parser_missing_month() {
        assert_eq!(year_month_parser().parse("1Y").unwrap(), (1, 0));
    }

    #[test]
    fn test_year_month_parser_zero_year() {
        assert_eq!(year_month_parser().parse("0Y2M").unwrap(), (0, 2));
    }

    #[test]
    fn test_year_month_parser_leading_zero() {
        assert_eq!(year_month_parser().parse("01Y02M").unwrap(), (1, 2));
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
}
