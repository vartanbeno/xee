use std::cmp::Ordering;

use crate::{error, Atomic};

pub(crate) trait AtomicCompare {
    fn atomic_compare<F>(
        a: Atomic,
        b: Atomic,
        string_compare: F,
        default_offset: chrono::FixedOffset,
    ) -> error::Result<bool>
    where
        F: Fn(&str, &str) -> Ordering;
}
