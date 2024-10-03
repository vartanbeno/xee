use ahash::AHashMap;
use std::fmt::Debug;

use xee_xpath_ast::ast;

use crate::error::Error;
use crate::sequence;

use super::dynamic_context_builder::StaticContextRef;
use super::DocumentsRef;

/// A map of variables
///
/// These are variables to be passed into an XPath evaluation.
///
/// The key is the name of a variable, and the value is an item.
pub type Variables = AHashMap<ast::Name, sequence::Sequence>;

// a dynamic context is created for each xpath evaluation
#[derive(Debug)]
pub struct DynamicContext<'a> {
    // we keep a reference to the static context. we don't need
    // to mutate it, and we want to be able create a new dynamic context from
    // the same static context quickly.
    pub static_context: StaticContextRef<'a>,

    /// An optional context item
    pub context_item: Option<sequence::Item>,
    // we want to mutate documents during evaluation, and this happens in
    // multiple spots. We use RefCell to manage that during runtime so we don't
    // need to make the whole thing immutable.
    pub documents: DocumentsRef,
    pub variables: Variables,
    // TODO: we want to be able to control the creation of this outside,
    // as it needs to be the same for all evalutions of XSLT I believe
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContext<'a> {
    pub(crate) fn new(
        static_context: StaticContextRef<'a>,
        context_item: Option<sequence::Item>,
        documents: DocumentsRef,
        variables: Variables,
        current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
    ) -> Self {
        Self {
            static_context,
            context_item,
            documents,
            variables,
            current_datetime,
        }
    }

    pub fn arguments(&self) -> Result<Vec<sequence::Sequence>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.static_context.parser_context.variable_names {
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

    pub(crate) fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.current_datetime.timezone()
    }

    pub fn documents(&self) -> DocumentsRef {
        self.documents.clone()
    }
}
