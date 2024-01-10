use ahash::HashSet;

use crate::ast;
use crate::{Namespaces, ParserError};

pub type VariableNames = HashSet<ast::Name>;

#[derive(Debug, Default)]
pub struct XPathParserContext<'a> {
    pub namespaces: Namespaces<'a>,
    pub variable_names: VariableNames,
}

impl<'a> XPathParserContext<'a> {
    /// Construct a new XPath parser context.
    ///
    /// This consists of information about namespaces and variable names
    /// available.
    pub fn new(namespaces: Namespaces<'a>, variable_names: VariableNames) -> Self {
        Self {
            namespaces,
            variable_names,
        }
    }

    /// Given an XPath string, parse into an XPath AST
    ///
    /// This uses the namespaces and variable names with which
    /// this static context has been initialized.
    pub fn parse_xpath(&self, s: &str) -> Result<ast::XPath, ParserError> {
        ast::XPath::parse(s, &self.namespaces, &self.variable_names)
    }

    /// Given an XSLT pattern, parse into an AST
    pub fn parse_pattern(&self, s: &str) -> Result<crate::Pattern, ParserError> {
        crate::Pattern::parse(s, &self.namespaces, &self.variable_names)
    }

    /// Parse an XPath string as it would appear in an XSLT value template.
    /// This means it should have a closing `}` following the xpath expression.
    pub fn parse_value_template_xpath(&self, s: &str) -> Result<ast::XPath, ParserError> {
        ast::XPath::parse_value_template(s, &self.namespaces, &self.variable_names)
    }
}
