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
    pub(crate) dynamic_context_builder:
        xee_interpreter::context::DynamicContextBuilder<'namespaces>,
    pub(crate) xot: Xot,
}

impl<'namespaces> Session<'namespaces> {
    pub(crate) fn new(
        queries: &'namespaces Queries<'namespaces>,
        documents: OwnedDocuments,
    ) -> Self {
        let mut dynamic_context_builder =
            xee_interpreter::context::DynamicContextBuilder::new(&queries.static_context);
        dynamic_context_builder.owned_documents(documents.documents.into_inner());
        Self {
            queries,
            dynamic_context_builder,
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
        MutableDocuments::new(&mut self.xot, self.dynamic_context_builder.get_documents())
    }

    pub fn documents(&self) -> RefDocuments {
        RefDocuments::new(&self.dynamic_context_builder.get_documents())
    }
}
