use std::cmp::Ordering;

use crate::{atomic::Atomic, error};

pub(crate) trait AtomicCompare {
    fn atomic_compare<F>(
        a: Atomic,
        b: Atomic,
        string_compare: F,
        default_offset: chrono::FixedOffset,
    ) -> error::Result<bool>
    where
        F: Fn(&str, &str) -> Ordering;

    // an inversion of the comparison is the same as the opposite comparison:
    // it's really when the arguments are inverted
    // so, a = b is the same as b = a
    // and a < b is the same as b > a
    // and a <= b is the same as b >= a
    fn arguments_inverted() -> impl AtomicCompare;
}
