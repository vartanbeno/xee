use crate::atomic::BinaryType;
use crate::error;
use crate::Atomic;

use super::cast_numeric::cast_numeric;
use super::datetime::EqWithDefaultOffset;

pub(crate) fn op_eq(
    a: Atomic,
    b: Atomic,
    default_offset: chrono::FixedOffset,
) -> error::Result<bool> {
    let (a, b) = cast_numeric(a, b)?;

    use Atomic::*;

    // cast guarantees both atomic types are the same concrete atomic
    match (a, b) {
        (Decimal(a), Decimal(b)) => Ok(a == b),
        (Integer(_, a), Integer(_, b)) => Ok(a == b),
        (Float(a), Float(b)) => Ok(a == b),
        (Double(a), Double(b)) => Ok(a == b),
        (Boolean(a), Boolean(b)) => Ok(a == b),
        (String(_, a), String(_, b)) => Ok(a == b),
        (Date(a), Date(b)) => Ok(a
            .as_ref()
            .eq_with_default_offset(b.as_ref(), default_offset)),
        (Time(a), Time(b)) => Ok(a
            .as_ref()
            .eq_with_default_offset(b.as_ref(), default_offset)),
        (DateTime(a), DateTime(b)) => Ok(a
            .as_ref()
            .eq_with_default_offset(b.as_ref(), default_offset)),
        (DateTimeStamp(a), DateTimeStamp(b)) => Ok(a == b),
        (Duration(a), Duration(b)) => Ok(a == b),
        (YearMonthDuration(a), YearMonthDuration(b)) => Ok(a == b),
        (DayTimeDuration(a), DayTimeDuration(b)) => Ok(a == b),
        (GYearMonth(a), GYearMonth(b)) => Ok(a == b),
        (GYear(a), GYear(b)) => Ok(a == b),
        (GMonthDay(a), GMonthDay(b)) => Ok(a == b),
        (GDay(a), GDay(b)) => Ok(a == b),
        (GMonth(a), GMonth(b)) => Ok(a == b),
        (Binary(BinaryType::Hex, a), Binary(BinaryType::Hex, b)) => Ok(a == b),
        (Binary(BinaryType::Base64, a), Binary(BinaryType::Base64, b)) => Ok(a == b),
        (QName(a), QName(b)) => Ok(a == b),
        _ => Err(error::Error::Type),
    }
}
