use std::cell::{RefCell, RefMut};

use xee_interpreter::context::Variables;
use xot::Xot;

use crate::{
    documents::{MutableDocuments, OwnedDocuments, RefDocuments},
    queries::Queries,
};

/// A session in which queries can be executed
///
/// You construct one using the [`Queries::session`] method.
#[derive(Debug)]
pub struct Session<'namespaces> {
    pub(crate) queries: &'namespaces Queries<'namespaces>,
    pub(crate) dynamic_context: xee_interpreter::context::DynamicContext<'namespaces>,
    pub(crate) xot: Xot,
}

impl<'namespaces> Session<'namespaces> {
    pub(crate) fn new(
        queries: &'namespaces Queries<'namespaces>,
        documents: OwnedDocuments,
    ) -> Self {
        let dynamic_context = xee_interpreter::context::DynamicContext::from_owned_documents(
            &queries.static_context,
            documents.documents,
            Variables::new(),
        );
        Self {
            queries,
            dynamic_context,
            xot: documents.xot,
        }
    }

    pub fn xot(&self) -> &Xot {
        &self.xot
    }

    pub fn xot_mut(&mut self) -> &mut Xot {
        &mut self.xot
    }

    pub fn documents_mut(&mut self) -> MutableDocuments {
        MutableDocuments::new(&mut self.xot, &self.dynamic_context.documents)
    }

    pub fn documents(&self) -> RefDocuments {
        RefDocuments::new(&self.dynamic_context.documents)
    }
}
