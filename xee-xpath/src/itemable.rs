use xee_interpreter::sequence::Item;

use crate::{error::Result, DocumentHandle, Documents};

// TODO: if the underlying APIs take a sequence we could turn this into
// a sequenceable.

/// Something that can be converted into an [`Item`] using a [`Document`]
///
/// This can be an item, but also a [`DocumentHandle`]
pub trait Itemable {
    /// Convert this itemable into an [`Item`]
    fn to_item(&self, documents: &Documents) -> Result<Item>;
}

impl Itemable for xot::Node {
    fn to_item(&self, _documents: &Documents) -> Result<Item> {
        Ok(Item::Node(*self))
    }
}

impl Itemable for DocumentHandle {
    fn to_item(&self, documents: &Documents) -> Result<Item> {
        // TODO: This unwrap is not great; we should turn this into an error
        let documents_ref = documents.documents.borrow();
        let document = documents_ref.get_by_handle(*self).unwrap();

        // getting the document element to start things off seems to
        // be the right expectation for the load API, though I'm not sure why
        // it shouldn't work with the root (document node) instead.
        let document_element = documents.xot().document_element(document.root()).unwrap();
        Ok(Item::Node(document_element))
    }
}

impl Itemable for &Item {
    fn to_item(&self, _documents: &Documents) -> Result<Item> {
        Ok(Clone::clone(self))
    }
}
