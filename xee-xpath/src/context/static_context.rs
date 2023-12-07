use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use ahash::HashSetExt;
use icu_provider_blob::BlobDataProvider;
use xee_xpath_ast::ast;
use xee_xpath_ast::Namespaces;

use crate::error;
use crate::function::StaticFunctions;
use crate::string::provider;
use crate::string::{Collation, Collations};

#[derive(Debug)]
pub struct StaticContext<'a> {
    pub(crate) namespaces: &'a Namespaces<'a>,
    // XXX need to add in type later
    pub(crate) variable_names: ast::VariableNames,
    pub(crate) functions: StaticFunctions,
    provider: BlobDataProvider,
    pub(crate) collations: RefCell<Collations>,
}

impl<'a> StaticContext<'a> {
    pub fn new(namespaces: &'a Namespaces<'a>) -> Self {
        Self {
            namespaces,
            variable_names: ast::VariableNames::new(),
            functions: StaticFunctions::new(),
            collations: RefCell::new(Collations::new()),
            provider: provider(),
        }
    }

    pub fn with_variable_names(
        namespaces: &'a Namespaces<'a>,
        variable_names: ast::VariableNames,
    ) -> Self {
        Self {
            namespaces,
            variable_names,
            functions: StaticFunctions::new(),
            collations: RefCell::new(Collations::new()),
            provider: provider(),
        }
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

    pub fn parse_xpath(&self, s: &str) -> Result<ast::XPath, xee_xpath_ast::ParserError> {
        ast::XPath::parse(s, self.namespaces, &self.variable_names)
    }
}
