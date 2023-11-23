use crate::ast_core::Span;
use crate::value_template;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct XmlName {
    pub namespace: String,
    pub local: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Error {
    // We didn't get the element we expect
    Unexpected,
    // Expected attribute of name, not found (element span)
    AttributeExpected {
        name: XmlName,
        span: Span,
    },
    // Did not expect attribute of name (attribute span)
    AttributeUnexpected {
        name: XmlName,
        span: Span,
        message: String,
    },
    // The value of the an attribute was invalid
    Invalid {
        value: String,
        span: Span,
    },

    UnexpectedSequenceConstructor,
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

    ElementMissing {
        span: Span,
    },

    UnexpectedElement {
        name: XmlName,
        span: Span,
    },
    ExpectedElementNotFound {
        expected: XmlName,
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
