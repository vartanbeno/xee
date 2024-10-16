// transform atomic nodes into PHP values
use ext_php_rs::{convert::IntoZval, types::ZendLong, types::Zval};

use ibig::IBig;
use xee_xpath::Atomic;

fn atomic_to_php(atomic: &Atomic, persistent: bool) -> Zval {
    match atomic {
        Atomic::Untyped(s) => s.as_ref().into_zval(persistent).unwrap(),
        Atomic::String(_, s) => s.as_ref().into_zval(persistent).unwrap(),
        Atomic::Float(f) => f.into_zval(persistent).unwrap(),
        Atomic::Double(d) => d.into_zval(persistent).unwrap(),
        // represent decimal as a PHP string
        Atomic::Decimal(d) => d.to_string().into_zval(persistent).unwrap(),
        Atomic::Integer(_, i) => {
            // try to turn it into a ZendLong first
            let l: Result<ZendLong, _> = i.as_ref().try_into();
            match l {
                Ok(l) => l.into_zval(persistent).unwrap(),
                Err(_) => {
                    // we can't fit it in an integer, so make it a float
                    i.to_f64().into_zval(persistent).unwrap()
                }
            }
        }
        Atomic::Duration(d) => {
            todo!()
        }
        Atomic::YearMonthDuration(d) => {
            todo!()
        }
        Atomic::DayTimeDuration(d) => {
            todo!()
        }
        Atomic::DateTime(dt) => {
            todo!()
        }
        Atomic::DateTimeStamp(ts) => {
            todo!()
        }
        Atomic::Time(t) => {
            todo!()
        }
        Atomic::Date(d) => {
            todo!()
        }
        Atomic::GYearMonth(ym) => {
            todo!()
        }
        Atomic::GYear(y) => {
            todo!()
        }
        Atomic::GMonthDay(md) => {
            todo!()
        }
        Atomic::GDay(d) => {
            todo!()
        }
        Atomic::GMonth(m) => {
            todo!()
        }
        Atomic::Boolean(b) => {
            todo!()
        }
        Atomic::Binary(_, b) => {
            todo!()
        }
        Atomic::QName(q) => {
            todo!()
        }
    }
}
