use xee_interpreter::sequence::Item;

use crate::{error::Result, DocumentHandle, Session};

/// Something that can be converted into an [`Item`] using a [`Session`]
///
/// This can be an item, but also a [`DocumentHandle`]
pub trait Itemable {
    /// Convert this itemable into an [`Item`]
    fn to_item(&self, session: &Session) -> Result<Item>;
}

impl Itemable for xot::Node {
    fn to_item(&self, _session: &Session) -> Result<Item> {
        Ok(Item::Node(*self))
    }
}

impl Itemable for DocumentHandle {
    fn to_item(&self, session: &Session) -> Result<Item> {
        assert!(self.documents_id == session.documents.id);
        let document_uri = &session.documents.document_uris[self.id];
        let borrowed_documents = session.dynamic_context.documents().borrow();
        let document = borrowed_documents.get(document_uri).unwrap();
        Ok(Item::Node(document.root()))
    }
}

impl Itemable for &Item {
    fn to_item(&self, _session: &Session) -> Result<Item> {
        Ok(Clone::clone(self))
    }
}
