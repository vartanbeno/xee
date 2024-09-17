use ahash::AHashMap;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;

use xee_xpath_ast::ast;

use crate::sequence;
use crate::xml;

use super::static_context::StaticContext;

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
    pub static_context: &'a StaticContext<'a>,
    // we want to mutate documents during evaluation, and this happens in
    // multiple spots. We use RefCell to manage that during runtime so we don't
    // need to make the whole thing immutable.
    pub documents: Cow<'a, RefCell<xml::Documents>>,
    // TODO: we want to be able to control the creation of this outside,
    // as it needs to be the same for all evalutions of XSLT I believe
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContext<'a> {
    pub fn new(
        static_context: &'a StaticContext<'a>,
        documents: Cow<'a, RefCell<xml::Documents>>,
    ) -> Self {
        Self {
            static_context,
            documents,
            current_datetime: Self::create_current_datetime(),
        }
    }

    pub fn from_documents(
        static_context: &'a StaticContext<'a>,
        documents: &'a RefCell<xml::Documents>,
    ) -> Self {
        Self::new(static_context, Cow::Borrowed(documents))
    }

    pub fn from_owned_documents(
        static_context: &'a StaticContext<'a>,
        documents: RefCell<xml::Documents>,
    ) -> Self {
        Self::new(static_context, Cow::Owned(documents))
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

    pub fn documents(&self) -> &RefCell<xml::Documents> {
        &self.documents
    }
}
