use std::rc::Rc;

use chrono::Offset;

use crate::{atomic::Atomic, error};

pub(crate) trait EqWithDefaultOffset: ToDateTimeStamp {
    fn eq_with_default_offset(&self, other: &Self, default_offset: chrono::FixedOffset) -> bool {
        let self_date_time_stamp = self.to_date_time_stamp(default_offset);
        let other_date_time_stamp = other.to_date_time_stamp(default_offset);
        self_date_time_stamp == other_date_time_stamp
    }
}

pub(crate) trait OrdWithDefaultOffset: ToDateTimeStamp {
    fn cmp_with_default_offset(
        &self,
        other: &Self,
        default_offset: chrono::FixedOffset,
    ) -> std::cmp::Ordering {
        let self_date_time_stamp = self.to_date_time_stamp(default_offset);
        let other_date_time_stamp = other.to_date_time_stamp(default_offset);
        self_date_time_stamp.cmp(&other_date_time_stamp)
    }
}

pub(crate) trait ToDateTimeStamp {
    fn to_date_time_stamp(
        &self,
        default_offset: chrono::FixedOffset,
    ) -> chrono::DateTime<chrono::FixedOffset>;
}

impl<T> EqWithDefaultOffset for T where T: ToDateTimeStamp {}
impl<T> OrdWithDefaultOffset for T where T: ToDateTimeStamp {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct YearMonthDuration {
    pub(crate) months: i64,
}

impl YearMonthDuration {
    pub(crate) fn new(months: i64) -> Self {
        Self { months }
    }

    pub(crate) fn years(&self) -> i64 {
        self.months / 12
    }

    pub(crate) fn months(&self) -> i64 {
        self.months % 12
    }
}

impl From<YearMonthDuration> for Atomic {
    fn from(year_month_duration: YearMonthDuration) -> Self {
        Atomic::YearMonthDuration(year_month_duration)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duration {
    pub(crate) year_month: YearMonthDuration,
    pub(crate) day_time: chrono::Duration,
}

impl Duration {
    pub(crate) fn new(months: i64, day_time: chrono::Duration) -> Self {
        Self {
            year_month: YearMonthDuration { months },
            day_time,
        }
    }

    pub(crate) fn from_year_month(year_month_duration: YearMonthDuration) -> Self {
        Self {
            year_month: year_month_duration,
            day_time: chrono::Duration::zero(),
        }
    }

    pub(crate) fn from_day_time(duration: chrono::Duration) -> Self {
        Self {
            year_month: YearMonthDuration { months: 0 },
            day_time: duration,
        }
    }
}

impl From<Duration> for Atomic {
    fn from(duration: Duration) -> Self {
        Atomic::Duration(Rc::new(duration))
    }
}

impl TryFrom<Atomic> for Duration {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Duration(d) => Ok(d.as_ref().clone()),
            Atomic::YearMonthDuration(d) => Ok(Duration::from_year_month(d)),
            Atomic::DayTimeDuration(d) => Ok(Duration::from_day_time(*d)),
            _ => Err(error::Error::Type),
        }
    }
}

impl TryFrom<Atomic> for chrono::Duration {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::DayTimeDuration(d) => Ok(*d.as_ref()),
            _ => Err(error::Error::Type),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NaiveDateTimeWithOffset {
    pub(crate) date_time: chrono::NaiveDateTime,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl From<NaiveDateTimeWithOffset> for chrono::DateTime<chrono::FixedOffset> {
    fn from(naive_date_time_with_offset: NaiveDateTimeWithOffset) -> Self {
        let offset = naive_date_time_with_offset
            .offset
            .unwrap_or_else(|| chrono::offset::Utc.fix());
        chrono::DateTime::from_utc(naive_date_time_with_offset.date_time, offset)
    }
}

impl From<chrono::DateTime<chrono::FixedOffset>> for NaiveDateTimeWithOffset {
    fn from(date_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
        NaiveDateTimeWithOffset::new(date_time.naive_utc(), Some(date_time.offset().clone()))
    }
}

impl From<NaiveDateTimeWithOffset> for Atomic {
    fn from(date_time: NaiveDateTimeWithOffset) -> Self {
        Atomic::DateTime(Rc::new(date_time))
    }
}

impl TryFrom<Atomic> for NaiveDateTimeWithOffset {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::DateTime(d) => Ok(d.as_ref().clone()),
            Atomic::DateTimeStamp(d) => Ok((*d.as_ref()).into()),
            _ => Err(error::Error::Type),
        }
    }
}

impl ToDateTimeStamp for NaiveDateTimeWithOffset {
    fn to_date_time_stamp(
        &self,
        default_offset: chrono::FixedOffset,
    ) -> chrono::DateTime<chrono::FixedOffset> {
        let offset = self.offset.unwrap_or(default_offset);
        chrono::DateTime::from_utc(self.date_time, offset)
    }
}

impl NaiveDateTimeWithOffset {
    pub(crate) fn new(
        date_time: chrono::NaiveDateTime,
        offset: Option<chrono::FixedOffset>,
    ) -> Self {
        Self { date_time, offset }
    }
}

#[derive(Debug, Clone)]
pub struct NaiveTimeWithOffset {
    pub(crate) time: chrono::NaiveTime,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl TryFrom<Atomic> for NaiveTimeWithOffset {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Time(d) => Ok(d.as_ref().clone()),
            _ => Err(error::Error::Type),
        }
    }
}

impl NaiveTimeWithOffset {
    pub(crate) fn new(time: chrono::NaiveTime, offset: Option<chrono::FixedOffset>) -> Self {
        Self { time, offset }
    }
}

impl ToDateTimeStamp for NaiveTimeWithOffset {
    fn to_date_time_stamp(
        &self,
        default_offset: chrono::FixedOffset,
    ) -> chrono::DateTime<chrono::FixedOffset> {
        let offset = self.offset.unwrap_or(default_offset);
        // https://www.w3.org/TR/xpath-functions-31/#func-subtract-times
        let date_time = chrono::NaiveDate::from_ymd_opt(1972, 12, 31)
            .unwrap()
            .and_time(self.time);
        // we need to get rid of the offset as we are going to add it
        // back next
        let date_time = date_time - offset;
        chrono::DateTime::from_utc(date_time, offset)
    }
}

impl From<NaiveTimeWithOffset> for Atomic {
    fn from(time: NaiveTimeWithOffset) -> Self {
        Atomic::Time(Rc::new(time))
    }
}

#[derive(Debug, Clone)]
pub struct NaiveDateWithOffset {
    pub(crate) date: chrono::NaiveDate,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl TryFrom<Atomic> for NaiveDateWithOffset {
    type Error = error::Error;

    fn try_from(a: Atomic) -> Result<Self, Self::Error> {
        match a {
            Atomic::Date(d) => Ok(d.as_ref().clone()),
            _ => Err(error::Error::Type),
        }
    }
}

impl NaiveDateWithOffset {
    pub(crate) fn new(date: chrono::NaiveDate, offset: Option<chrono::FixedOffset>) -> Self {
        Self { date, offset }
    }
}

impl ToDateTimeStamp for NaiveDateWithOffset {
    fn to_date_time_stamp(
        &self,
        default_offset: chrono::FixedOffset,
    ) -> chrono::DateTime<chrono::FixedOffset> {
        let offset = self.offset.unwrap_or(default_offset);
        let date_time = self.date.and_hms_opt(0, 0, 0).unwrap();
        chrono::DateTime::from_utc(date_time, offset)
    }
}

impl From<NaiveDateWithOffset> for Atomic {
    fn from(date: NaiveDateWithOffset) -> Self {
        Atomic::Date(Rc::new(date))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GYearMonth {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl GYearMonth {
    pub(crate) fn new(year: i32, month: u32, offset: Option<chrono::FixedOffset>) -> Self {
        Self {
            year,
            month,
            offset,
        }
    }
}

impl From<GYearMonth> for Atomic {
    fn from(g_year_month: GYearMonth) -> Self {
        Atomic::GYearMonth(Rc::new(g_year_month))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GYear {
    pub(crate) year: i32,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl GYear {
    pub(crate) fn new(year: i32, offset: Option<chrono::FixedOffset>) -> Self {
        Self { year, offset }
    }
}

impl From<GYear> for Atomic {
    fn from(g_year: GYear) -> Self {
        Atomic::GYear(Rc::new(g_year))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMonthDay {
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl GMonthDay {
    pub(crate) fn new(month: u32, day: u32, offset: Option<chrono::FixedOffset>) -> Self {
        Self { month, day, offset }
    }
}

impl From<GMonthDay> for Atomic {
    fn from(g_month_day: GMonthDay) -> Self {
        Atomic::GMonthDay(Rc::new(g_month_day))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GDay {
    pub(crate) day: u32,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl GDay {
    pub(crate) fn new(day: u32, offset: Option<chrono::FixedOffset>) -> Self {
        Self { day, offset }
    }
}

impl From<GDay> for Atomic {
    fn from(g_day: GDay) -> Self {
        Atomic::GDay(Rc::new(g_day))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GMonth {
    pub(crate) month: u32,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl GMonth {
    pub(crate) fn new(month: u32, offset: Option<chrono::FixedOffset>) -> Self {
        Self { month, offset }
    }
}

impl From<GMonth> for Atomic {
    fn from(g_month: GMonth) -> Self {
        Atomic::GMonth(Rc::new(g_month))
    }
}

impl From<chrono::Duration> for Atomic {
    fn from(duration: chrono::Duration) -> Self {
        Atomic::DayTimeDuration(Rc::new(duration))
    }
}

impl From<chrono::DateTime<chrono::FixedOffset>> for Atomic {
    fn from(date_time: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Atomic::DateTimeStamp(Rc::new(date_time))
    }
}
