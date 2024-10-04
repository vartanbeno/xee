use xot::Xot;

use xee_interpreter::context::DocumentsRef;

use crate::documents::Documents;

/// A session in which queries can be executed
///
/// You construct one using the [`Queries::session`] method.
#[derive(Debug)]
pub struct Session {
    pub(crate) documents: DocumentsRef,
    pub(crate) xot: Xot,
}

impl Session {
    pub(crate) fn new(documents: DocumentsRef, xot: Xot) -> Self {
        Self { documents, xot }
    }

    pub(crate) fn from_documents(documents: Documents) -> Self {
        Self::new(documents.documents, documents.xot)
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
