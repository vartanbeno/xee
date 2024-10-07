use xot::Xot;

use xee_interpreter::context::DocumentsRef;

use crate::documents::Documents;

/// A session in which queries can be executed
///
/// You construct one using the [`Queries::session`] method.
#[derive(Debug)]
pub struct Session<'a> {
    pub(crate) documents: DocumentsRef,
    pub(crate) xot: &'a mut Xot,
}

impl<'a> Session<'a> {
    // TODO: public for now, but should make it private once we have
    // a good way to construct session from a dynamic context or something.
    pub fn new(documents: DocumentsRef, xot: &'a mut Xot) -> Self {
        Self { documents, xot }
    }

    pub(crate) fn from_documents(documents: &'a mut Documents) -> Self {
        Self::new(documents.documents.clone(), &mut documents.xot)
    }

    /// Get a reference to the Xot arena
    pub fn xot(&self) -> &Xot {
        self.xot
    }

    /// Get a mutable reference to the Xot arena
    pub fn xot_mut(&mut self) -> &mut Xot {
        self.xot
    }
}
