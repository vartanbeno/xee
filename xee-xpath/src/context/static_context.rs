use std::fmt::Debug;

use xee_xpath_ast::ast;
use xee_xpath_ast::Namespaces;

use super::static_function::StaticFunctions;

#[derive(Debug)]
pub struct StaticContext<'a> {
    pub(crate) namespaces: &'a Namespaces<'a>,
    // XXX need to add in type later
    pub(crate) variables: Vec<ast::Name>,
    pub(crate) functions: StaticFunctions,
}

impl<'a> StaticContext<'a> {
    pub fn new(namespaces: &'a Namespaces<'a>) -> Self {
        Self {
            namespaces,
            variables: Vec::new(),
            functions: StaticFunctions::new(namespaces),
        }
    }

    pub fn with_variable_names(namespaces: &'a Namespaces<'a>, variables: &[ast::Name]) -> Self {
        Self {
            namespaces,
            variables: variables.to_vec(),
            functions: StaticFunctions::new(namespaces),
        }
    }
}
