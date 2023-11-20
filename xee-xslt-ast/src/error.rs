use crate::ast_core::Span;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Error {
    Unexpected,
    // ns, name, span of the element on which the attribute is expected
    AttributeExpected {
        namespace: String,
        local: String,
        span: Span,
    },
    // ns, name, span of the attribute that is unexpected
    AttributeUnexpected {
        namespace: String,
        local: String,
        span: Span,
        message: String,
    },
    UnexpectedSequenceConstructor,
    Invalid {
        value: String,
        span: Span,
    },
    InvalidInstruction {
        span: Span,
    },
    MissingSpan,
    XPath(xee_xpath_ast::ParserError),
}

impl From<xee_xpath_ast::ParserError> for Error {
    fn from(error: xee_xpath_ast::ParserError) -> Self {
        Self::XPath(error)
    }
}

impl Error {
    pub(crate) fn is_unexpected(&self) -> bool {
        matches!(self, Self::Unexpected)
    }
}
