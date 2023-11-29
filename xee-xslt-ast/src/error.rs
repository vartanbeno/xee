use crate::{ast_core::Span, name::XmlName, value_template};

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AttributeError {
    // Expected attribute of name, not found (element span)
    NotFound { name: XmlName, span: Span },
    // Did not expect attribute of name (attribute span)
    Unexpected { name: XmlName, span: Span },
    // The value of an attribute was invalid
    Invalid { value: String, span: Span },
    // An eqname was invalid
    InvalidEqName { value: String, span: Span },
    // XPath parser error
    XPath(xee_xpath_ast::ParserError),
    // A value templatecould not be parsed
    ValueTemplateError(value_template::Error),
    // Internal error; should not happen
    Internal,
}

impl From<xee_xpath_ast::ParserError> for AttributeError {
    fn from(e: xee_xpath_ast::ParserError) -> Self {
        AttributeError::XPath(e)
    }
}

impl From<value_template::Error> for AttributeError {
    fn from(e: value_template::Error) -> Self {
        AttributeError::ValueTemplateError(e)
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub(crate) enum ElementError {
    // Did not expect this node
    Unexpected { span: Span },
    // Did not expect end TODO: how to get span info?
    UnexpectedEnd,
    // An attribute of the element was invalid
    Attribute(AttributeError),

    // internal error, should not happen
    Internal,
}

impl From<AttributeError> for ElementError {
    fn from(error: AttributeError) -> Self {
        Self::Attribute(error)
    }
}

// type Result<T> = std::result::Result<T, ElementError>;
