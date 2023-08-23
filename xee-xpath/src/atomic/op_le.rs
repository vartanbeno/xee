use crate::atomic::datetime::OrdWithDefaultOffset;
use crate::atomic::BinaryType;
use crate::error;
use crate::Atomic;

use super::cast_binary::cast_binary_compare;

pub(crate) fn op_le(
    a: Atomic,
    b: Atomic,
    default_offset: chrono::FixedOffset,
) -> error::Result<bool> {
    let (a, b) = cast_binary_compare(a, b)?;

    use Atomic::*;

    match (a, b) {
        (Decimal(a), Decimal(b)) => Ok(a <= b),
        (Integer(_, a), Integer(_, b)) => Ok(a <= b),
        (Float(a), Float(b)) => Ok(a <= b),
        (Double(a), Double(b)) => Ok(a <= b),
        (Boolean(a), Boolean(b)) => Ok(a <= b),
        (String(_, a), String(_, b)) => Ok(a <= b),
        (Date(a), Date(b)) => Ok(a
            .as_ref()
            .cmp_with_default_offset(b.as_ref(), default_offset)
            .is_le()),
        (Time(a), Time(b)) => Ok(a
            .as_ref()
            .cmp_with_default_offset(b.as_ref(), default_offset)
            .is_le()),
        (DateTime(a), DateTime(b)) => Ok(a
            .as_ref()
            .cmp_with_default_offset(b.as_ref(), default_offset)
            .is_le()),
        (DateTimeStamp(a), DateTimeStamp(b)) => Ok(a <= b),
        (YearMonthDuration(a), YearMonthDuration(b)) => Ok(a <= b),
        (DayTimeDuration(a), DayTimeDuration(b)) => Ok(a <= b),
        (Binary(BinaryType::Hex, a), Binary(BinaryType::Hex, b)) => Ok(a <= b),
        (Binary(BinaryType::Base64, a), Binary(BinaryType::Base64, b)) => Ok(a <= b),
        _ => Err(error::Error::Type),
    }
}
