// transform atomic nodes into PHP values
use ext_php_rs::{convert::IntoZval, types::ZendLong, types::Zval};

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
                // the ibig is too big
                Err(_) => {
                    // we can't fit it in an integer, so make it a float
                    i.to_f64().into_zval(persistent).unwrap()
                }
            }
        }
        // all the duration types may be put into a DateInterval
        Atomic::Duration(d) => {
            todo!()
        }
        Atomic::YearMonthDuration(d) => {
            todo!()
        }
        Atomic::DayTimeDuration(d) => {
            todo!()
        }
        // can we put all the datetime types into a DateTimeImmutable?
        Atomic::DateTime(dt) => {
            todo!()
        }
        Atomic::DateTimeStamp(ts) => {
            todo!()
        }
        // is it possible to represent time? could we use DateInterval?
        Atomic::Time(t) => {
            todo!()
        }
        Atomic::Date(d) => {
            todo!()
        }
        // it probably makes sense to create PHP classes for all the G* stuff
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
        Atomic::Boolean(b) => b.into_zval(persistent).unwrap(),
        Atomic::Binary(_, b) => {
            // TODO: ext_php_rs doesn't have a way to turn a u8 slice into a
            // PHP string, so we need to implement that
            todo!()
        }
        // it probably makes sense to make a PHP class to represent QName
        Atomic::QName(q) => {
            todo!()
        }
    }
}
