use chrono::Offset;

use crate::context::DynamicContext;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct YearMonthDuration {
    pub(crate) months: i64,
}

impl YearMonthDuration {
    pub(crate) fn new(months: i64) -> Self {
        Self { months }
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Ord for NaiveDateTimeWithOffset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_date_time_stamp = self.to_date_time_stamp();
        let other_date_time_stamp = other.to_date_time_stamp();
        self_date_time_stamp.cmp(&other_date_time_stamp)
    }
}

impl PartialOrd for NaiveDateTimeWithOffset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl NaiveDateTimeWithOffset {
    pub(crate) fn new(
        date_time: chrono::NaiveDateTime,
        offset: Option<chrono::FixedOffset>,
    ) -> Self {
        Self { date_time, offset }
    }

    fn to_date_time_stamp(&self) -> chrono::DateTime<chrono::FixedOffset> {
        let offset = self.offset.unwrap_or_else(|| chrono::offset::Utc.fix());
        chrono::DateTime::from_utc(self.date_time, offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaiveTimeWithOffset {
    pub(crate) time: chrono::NaiveTime,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl NaiveTimeWithOffset {
    pub(crate) fn new(time: chrono::NaiveTime, offset: Option<chrono::FixedOffset>) -> Self {
        Self { time, offset }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaiveDateWithOffset {
    pub(crate) date: chrono::NaiveDate,
    pub(crate) offset: Option<chrono::FixedOffset>,
}

impl NaiveDateWithOffset {
    pub(crate) fn new(date: chrono::NaiveDate, offset: Option<chrono::FixedOffset>) -> Self {
        Self { date, offset }
    }

    fn to_date_time_stamp(&self) -> chrono::DateTime<chrono::FixedOffset> {
        let offset = self.offset.unwrap_or_else(|| chrono::offset::Utc.fix());
        let date_time = self.date.and_hms_opt(0, 0, 0).unwrap();
        chrono::DateTime::from_utc(date_time, offset)
    }
}

impl Ord for NaiveDateWithOffset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_date_time_stamp = self.to_date_time_stamp();
        let other_date_time_stamp = other.to_date_time_stamp();
        self_date_time_stamp.cmp(&other_date_time_stamp)
    }
}

impl PartialOrd for NaiveDateWithOffset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
