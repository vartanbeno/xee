use xot::Xot;

use xee_interpreter::context::DocumentsRef;

use crate::{documents::Documents, queries::Queries};

/// A session in which queries can be executed
///
/// You construct one using the [`Queries::session`] method.
#[derive(Debug)]
pub struct Session<'namespaces> {
    pub(crate) queries: &'namespaces Queries<'namespaces>,
    pub(crate) documents: DocumentsRef,
    pub(crate) xot: Xot,
}

impl<'namespaces> Session<'namespaces> {
    pub(crate) fn new(queries: &'namespaces Queries, documents: DocumentsRef, xot: Xot) -> Self {
        Self {
            queries,
            documents,
            xot,
        }
    }

    pub(crate) fn from_documents(queries: &'namespaces Queries, documents: Documents) -> Self {
        Self::new(queries, documents.documents, documents.xot)
    }

    /// Get a reference to the Xot arena
    pub fn xot(&self) -> &Xot {
        &self.xot
    }

    /// Get a mutable reference to the Xot arena
    pub fn xot_mut(&mut self) -> &mut Xot {
        &mut self.xot
    }
}
