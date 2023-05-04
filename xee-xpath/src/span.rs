use miette::SourceSpan;

pub(crate) type Spanned<T> = (T, SourceSpan);

pub(crate) fn not_spanned<T>(value: T) -> Spanned<T> {
    (value, (0, 0).into())
}
