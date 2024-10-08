use ahash::AHashMap;
use std::fmt::Debug;

use xee_xpath_ast::ast;

use crate::function::Function;
use crate::{error::Error, interpreter::Program};
use crate::{interpreter, sequence};

use super::{DocumentsRef, StaticContext};

/// A map of variables
///
/// These are variables to be passed into an XPath evaluation.
///
/// The key is the name of a variable, and the value is an item.
pub type Variables = AHashMap<ast::Name, sequence::Sequence>;

// a dynamic context is created for each xpath evaluation
#[derive(Debug)]
pub struct DynamicContext<'a> {
    // we keep a reference to the program
    program: &'a Program,

    /// An optional context item
    context_item: Option<sequence::Item>,
    // we want to mutate documents during evaluation, and this happens in
    // multiple spots. We use RefCell to manage that during runtime so we don't
    // need to make the whole thing immutable.
    documents: DocumentsRef,
    variables: Variables,
    // TODO: we want to be able to control the creation of this outside,
    // as it needs to be the same for all evalutions of XSLT I believe
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContext<'a> {
    pub(crate) fn new(
        program: &'a Program,
        context_item: Option<sequence::Item>,
        documents: DocumentsRef,
        variables: Variables,
        current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
    ) -> Self {
        Self {
            program,
            context_item,
            documents,
            variables,
            current_datetime,
        }
    }

    /// The static context of the program.
    pub fn static_context(&self) -> &StaticContext {
        self.program.static_context()
    }

    /// Access the context item, if any.
    pub fn context_item(&self) -> Option<&sequence::Item> {
        self.context_item.as_ref()
    }

    /// The documents in this context.
    pub fn documents(&self) -> DocumentsRef {
        self.documents.clone()
    }

    /// The variables in this context.
    pub fn variables(&self) -> &Variables {
        &self.variables
    }

    pub(crate) fn arguments(&self) -> Result<Vec<sequence::Sequence>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.program.static_context().parser_context.variable_names {
            let items = self.variables.get(variable_name).ok_or(Error::XPDY0002)?;
            arguments.push(items.clone());
        }
        Ok(arguments)
    }

    fn create_current_datetime() -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::offset::Local::now().into()
    }

    pub(crate) fn current_datetime(&self) -> chrono::DateTime<chrono::offset::FixedOffset> {
        self.current_datetime
    }

    pub fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.current_datetime.timezone()
    }

    /// Access information about a Function.
    pub fn function_info<'b>(&self, function: &'b Function) -> interpreter::FunctionInfo<'a, 'b> {
        self.program.function_info(function)
    }
}
