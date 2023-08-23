use crate::error;
use crate::Atomic;

use super::cast_binary::cast_binary_compare;
use super::op_eq::op_eq;

pub(crate) fn op_ne(
    a: Atomic,
    b: Atomic,
    default_offset: chrono::FixedOffset,
) -> error::Result<bool> {
    let (a, b) = cast_binary_compare(a, b)?;

    op_eq(a, b, default_offset).map(|eq| !eq)
}
