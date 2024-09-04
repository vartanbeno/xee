use ahash::AHashMap;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;

use xee_xpath_ast::ast;

use crate::error::Error;
use crate::sequence;
use crate::xml;

use super::static_context::StaticContext;

pub type Variables = AHashMap<ast::Name, sequence::Sequence>;

// a dynamic context is created for each xpath evaluation
#[derive(Debug)]
pub struct DynamicContext<'a> {
    // we keep a reference to the static context. we don't need
    // to mutate it, and we want to be able create a new dynamic context from
    // the same static context quickly.
    pub static_context: &'a StaticContext<'a>,
    // we want to mutate documents during evaluation, and this happens in
    // multiple spots. We use RefCell to manage that during runtime so we don't
    // need to make the whole thing immutable.
    pub documents: &'a RefCell<xml::Documents>,
    // the variables is either a reference or owned. variables are immutable.
    // a reference is handy if we have no variables so we don't need to
    // recreate them each time.
    pub(crate) variables: Cow<'a, Variables>,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContext<'a> {
    pub fn new(
        static_context: &'a StaticContext<'a>,
        documents: &'a RefCell<xml::Documents>,
        variables: Cow<'a, Variables>,
    ) -> Self {
        Self {
            static_context,
            documents,
            variables,
            current_datetime: Self::create_current_datetime(),
        }
    }

    pub fn from_documents(
        static_context: &'a StaticContext<'a>,
        documents: &'a RefCell<xml::Documents>,
    ) -> Self {
        Self::new(static_context, documents, Cow::Owned(Variables::default()))
    }

    pub fn from_variables(
        static_context: &'a StaticContext<'a>,
        documents: &'a RefCell<xml::Documents>,
        variables: Cow<'a, Variables>,
    ) -> Self {
        Self::new(static_context, documents, variables)
    }

    fn create_current_datetime() -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::offset::Local::now().into()
    }

    pub fn arguments(&self) -> Result<Vec<sequence::Sequence>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.static_context.parser_context.variable_names {
            let items = self.variables.get(variable_name).ok_or(Error::XPDY0002)?;
            arguments.push(items.clone());
        }
        Ok(arguments)
    }

    pub(crate) fn current_datetime(&self) -> chrono::DateTime<chrono::offset::FixedOffset> {
        self.current_datetime
    }

    pub(crate) fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.current_datetime.timezone()
    }

    pub fn documents(&self) -> &RefCell<xml::Documents> {
        &self.documents
    }
}
