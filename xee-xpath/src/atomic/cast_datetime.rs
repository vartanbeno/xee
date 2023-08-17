use rust_decimal::prelude::*;

use crate::atomic;
use crate::error;

impl atomic::Atomic {
    pub(crate) fn canonical_duration(months: i32, duration: chrono::Duration) -> String {
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

    pub(crate) fn canonical_year_month_duration(months: i32) -> String {
        let mut s = String::new();
        if months < 0 {
            s.push('-');
        }
        s.push('P');
        Self::push_canonical_year_month_duration_fragment(&mut s, months);
        s
    }

    fn push_canonical_year_month_duration_fragment(s: &mut String, months: i32) {
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
}
