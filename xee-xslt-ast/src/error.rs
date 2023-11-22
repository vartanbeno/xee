use crate::ast_core::Span;
use crate::value_template;

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
    InvalidValueTemplate {
        span: Span,
    },

    MissingSpan,
    InvalidEqName {
        value: String,
        span: Span,
    },
    /// An internal error; this indicates a bug as some invariant in the
    /// code wasn't met.
    Internal(&'static str),
    XPath(xee_xpath_ast::ParserError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<xee_xpath_ast::ParserError> for Error {
    fn from(error: xee_xpath_ast::ParserError) -> Self {
        Self::XPath(error)
    }
}

impl From<value_template::Error> for Error {
    fn from(error: value_template::Error) -> Self {
        match error {
            value_template::Error::UnescapedCurly { span, .. } => {
                Self::InvalidValueTemplate { span }
            }
            value_template::Error::IllegalSlice => Self::Internal("Illegal slice"),
            value_template::Error::XPath(e) => Self::XPath(e),
        }
    }
}

impl Error {
    pub(crate) fn is_unexpected(&self) -> bool {
        matches!(self, Self::Unexpected)
    }
}
