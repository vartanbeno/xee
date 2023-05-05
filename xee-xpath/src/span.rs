use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Spanned<T> {
    pub(crate) value: T,
    pub(crate) span: SourceSpan,
}

impl<T> Spanned<T> {
    pub(crate) fn new(value: T, span: SourceSpan) -> Self {
        Self { value, span }
    }
}

// pub(crate) type Spanned<T> = (T, SourceSpan);

pub(crate) fn not_spanned<T>(value: T) -> Spanned<T> {
    Spanned {
        value,
        span: (0, 0).into(),
    }
}
