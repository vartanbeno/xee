use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use icu_provider_blob::BlobDataProvider;
use xee_xpath_ast::ast;
use xee_xpath_ast::Namespaces;
use xee_xpath_ast::VariableNames;
use xee_xpath_ast::XPathParserContext;

use crate::error;
use crate::function::StaticFunctions;
use crate::string::provider;
use crate::string::{Collation, Collations};

#[derive(Debug)]
pub struct StaticContext<'a> {
    pub(crate) parser_context: XPathParserContext<'a>,
    pub(crate) functions: StaticFunctions,
    provider: BlobDataProvider,
    pub(crate) collations: RefCell<Collations>,
}

impl<'a> Default for StaticContext<'a> {
    fn default() -> Self {
        Self::new(Namespaces::default(), VariableNames::default())
    }
}

impl<'a> StaticContext<'a> {
    pub fn new(namespaces: Namespaces<'a>, variable_names: VariableNames) -> Self {
        Self {
            parser_context: XPathParserContext::new(namespaces, variable_names),
            functions: StaticFunctions::new(),
            collations: RefCell::new(Collations::new()),
            provider: provider(),
        }
    }

    pub fn from_namespaces(namespaces: Namespaces<'a>) -> Self {
        Self::new(namespaces, VariableNames::default())
    }

    pub fn namespaces(&self) -> &Namespaces {
        &self.parser_context.namespaces
    }

    pub fn variable_names(&self) -> &VariableNames {
        &self.parser_context.variable_names
    }

    pub(crate) fn default_collation(&self) -> error::Result<Rc<Collation>> {
        self.collation(self.default_collation_uri())
    }

    pub(crate) fn default_collation_uri(&self) -> &str {
        "http://www.w3.org/2005/xpath-functions/collation/codepoint"
    }

    pub(crate) fn collation(&self, uri: &str) -> error::Result<Rc<Collation>> {
        // TODO: supply static base URI
        self.collations
            .borrow_mut()
            .load(self.provider.clone(), None, uri)
    }

    pub(crate) fn icu_provider(&self) -> &BlobDataProvider {
        &self.provider
    }

    /// Given an XPath string, parse into an XPath AST
    ///
    /// This uses the namespaces and variable names with which
    /// this static context has been initialized.
    pub fn parse_xpath(&self, s: &str) -> Result<ast::XPath, xee_xpath_ast::ParserError> {
        self.parser_context.parse_xpath(s)
    }

    /// Parse an XPath string as it would appear in an XSLT value template.
    /// This means it should have a closing `}` following the xpath expression.
    pub fn parse_value_template_xpath(
        &self,
        s: &str,
    ) -> Result<ast::XPath, xee_xpath_ast::ParserError> {
        self.parser_context.parse_value_template_xpath(s)
    }
}
