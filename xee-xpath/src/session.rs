use crate::{
    documents::{Documents, InnerDocuments},
    queries::Queries,
};

/// A session in which queries can be executed
///
/// You construct one using the [`Queries::session`] method.
#[derive(Debug)]
pub struct Session<'namespaces> {
    pub(crate) queries: &'namespaces Queries<'namespaces>,
    pub(crate) dynamic_context: xee_interpreter::context::DynamicContext<'namespaces>,
    pub(crate) documents: InnerDocuments,
}

impl<'namespaces> Session<'namespaces> {
    pub(crate) fn new(queries: &'namespaces Queries<'namespaces>, documents: Documents) -> Self {
        let dynamic_context = xee_interpreter::context::DynamicContext::from_owned_documents(
            &queries.static_context,
            documents.documents,
        );
        Self {
            queries,
            dynamic_context,
            documents: documents.inner,
        }
    }
}
