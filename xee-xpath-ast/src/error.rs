use chumsky::input::ValueInput;
use chumsky::prelude::SimpleSpan as Span;
use chumsky::util::MaybeRef;

use crate::ast;
use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParserError<'a> {
    ExpectedFound {
        span: Span,
        expected: Vec<Option<Token<'a>>>,
        found: Option<Token<'a>>,
    },
    UnknownPrefix {
        span: Span,
        prefix: String,
    },
    Reserved {
        span: Span,
        name: String,
    },
    ArityOverflow {
        span: Span,
    },
    UnknownType {
        span: Span,
        name: ast::Name,
    },
}

impl<'a> ParserError<'a> {
    pub fn span(&self) -> Span {
        match self {
            Self::ExpectedFound { span, .. } => *span,
            Self::UnknownPrefix { span, .. } => *span,
            Self::Reserved { span, .. } => *span,
            Self::ArityOverflow { span } => *span,
            Self::UnknownType { span, .. } => *span,
        }
    }
}

impl<'a, I> chumsky::error::Error<'a, I> for ParserError<'a>
where
    I: ValueInput<'a, Token = Token<'a>, Span = Span>,
{
    fn expected_found<E: IntoIterator<Item = Option<MaybeRef<'a, Token<'a>>>>>(
        expected: E,
        found: Option<MaybeRef<'a, Token<'a>>>,
        span: Span,
    ) -> Self {
        Self::ExpectedFound {
            span,
            expected: expected
                .into_iter()
                .map(|e| e.as_deref().cloned())
                .collect(),
            found: found.as_deref().cloned(),
        }
    }

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (
                ParserError::ExpectedFound {
                    expected: a,
                    span: span_a,
                    found: found_a,
                },
                ParserError::ExpectedFound {
                    expected: b,
                    span: _,
                    found: _,
                },
            ) => {
                let mut combined = Vec::new();
                for a_entry in a.into_iter() {
                    combined.push(a_entry);
                }
                for b_entry in b.into_iter() {
                    if !combined.contains(&b_entry) {
                        combined.push(b_entry);
                    }
                }
                ParserError::ExpectedFound {
                    span: span_a,
                    expected: combined,
                    found: found_a,
                }
            }
            (ParserError::ExpectedFound { .. }, a) => a,
            (a, ParserError::ExpectedFound { .. }) => a,
            (a, _) => a,
        }
    }
}

