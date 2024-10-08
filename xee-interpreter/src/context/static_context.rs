use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::LazyLock;

use xee_name::{Namespaces, VariableNames};
use xee_xpath_ast::ast;
use xee_xpath_ast::XPathParserContext;

use crate::error;
use crate::function;
use crate::string::{Collation, Collations};

static STATIC_FUNCTIONS: LazyLock<function::StaticFunctions> =
    LazyLock::new(function::StaticFunctions::new);

#[derive(Debug)]
pub struct StaticContext {
    parser_context: XPathParserContext,
    functions: &'static function::StaticFunctions,
    // TODO: try to make collations static
    collations: RefCell<Collations>,
}

impl Default for StaticContext {
    fn default() -> Self {
        Self::new(Namespaces::default(), VariableNames::default())
    }
}

impl From<XPathParserContext> for StaticContext {
    fn from(parser_context: XPathParserContext) -> Self {
        Self {
            parser_context,
            functions: &STATIC_FUNCTIONS,
            collations: RefCell::new(Collations::new()),
        }
    }
}

impl StaticContext {
    pub(crate) fn new(namespaces: Namespaces, variable_names: VariableNames) -> Self {
        Self {
            parser_context: XPathParserContext::new(namespaces, variable_names),
            functions: &STATIC_FUNCTIONS,
            collations: RefCell::new(Collations::new()),
        }
    }

    pub fn from_namespaces(namespaces: Namespaces) -> Self {
        Self::new(namespaces, VariableNames::default())
    }

    pub fn namespaces(&self) -> &Namespaces {
        &self.parser_context.namespaces
    }

    pub fn variable_names(&self) -> &VariableNames {
        &self.parser_context.variable_names
    }

    pub fn default_collation(&self) -> error::Result<Rc<Collation>> {
        self.collation(self.default_collation_uri())
    }

    pub fn default_collation_uri(&self) -> &str {
        "http://www.w3.org/2005/xpath-functions/collation/codepoint"
    }

    pub(crate) fn collation(&self, uri: &str) -> error::Result<Rc<Collation>> {
        // TODO: supply static base URI
        self.collations.borrow_mut().load(None, uri)
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

    /// Get a static function by id
    pub fn function_by_id(
        &self,
        static_function_id: function::StaticFunctionId,
    ) -> &function::StaticFunction {
        self.functions.get_by_index(static_function_id)
    }

    /// Get a static function by name and arity
    pub fn function_id_by_name(
        &self,
        name: &xot::xmlname::OwnedName,
        arity: u8,
    ) -> Option<function::StaticFunctionId> {
        self.functions.get_by_name(name, arity)
    }
}
