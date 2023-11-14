use chumsky::input::ValueInput;
use chumsky::prelude::SimpleSpan as Span;
use chumsky::util::MaybeRef;

use crate::ast;
use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParserError {
    ExpectedFound { span: Span },
    UnknownPrefix { span: Span, prefix: String },
    Reserved { span: Span, name: String },
    ArityOverflow { span: Span },
    UnknownType { span: Span, name: ast::Name },
}

impl ParserError {
    pub fn span(&self) -> Span {
        match self {
            Self::ExpectedFound { span, .. } => *span,
            Self::UnknownPrefix { span, .. } => *span,
            Self::Reserved { span, .. } => *span,
            Self::ArityOverflow { span } => *span,
            Self::UnknownType { span, .. } => *span,
        }
    }

    pub fn adjust(mut self, start: usize) -> Self {
        use ParserError::*;
        let span = match &mut self {
            ExpectedFound { span } => span,
            UnknownPrefix { span, .. } => span,
            Reserved { span, .. } => span,
            ArityOverflow { span } => span,
            UnknownType { span, .. } => span,
        };
        *span = Span::new(span.start + start, span.end + start);
        self
    }
}

impl<'a, I> chumsky::error::Error<'a, I> for ParserError
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    // we don't do anything with expected and found, instead just retaining
    // the span. This is because these contain tokens with a lifetime, and
    // having a lifetime for the ParserError turns out open up a world of trouble
    // as soon as we want to build on it in the XSLT parser. We also don't
    // have a good way to turn a logos token into a human-readable string, so
    // we couldn't really construct good error messages anyway.
    fn expected_found<E: IntoIterator<Item = Option<MaybeRef<'a, Token<'a>>>>>(
        _expected: E,
        _found: Option<MaybeRef<'a, Token<'a>>>,
        span: Span,
    ) -> Self {
        Self::ExpectedFound { span }
    }

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (
                ParserError::ExpectedFound { span: span_a },
                ParserError::ExpectedFound { span: _ },
            ) => ParserError::ExpectedFound { span: span_a },
            (ParserError::ExpectedFound { .. }, a) => a,
            (a, ParserError::ExpectedFound { .. }) => a,
            (a, _) => a,
        }
    }
}
