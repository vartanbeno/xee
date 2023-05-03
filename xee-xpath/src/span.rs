use std::ops::Range;

pub(crate) type Spanned<T> = (T, Span);
pub(crate) type Span = Range<usize>;

pub(crate) fn not_spanned<T>(value: T) -> Spanned<T> {
    (value, (0..0))
}
