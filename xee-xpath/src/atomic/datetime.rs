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

impl NaiveDateTimeWithOffset {
    pub(crate) fn new(
        date_time: chrono::NaiveDateTime,
        offset: Option<chrono::FixedOffset>,
    ) -> Self {
        Self { date_time, offset }
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
