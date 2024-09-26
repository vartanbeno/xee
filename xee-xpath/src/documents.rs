use std::cell::RefCell;

use xee_interpreter::xml::{Document, DocumentHandle, DocumentsError, Uri};
use xot::Xot;

/// A collection of XML documents as can be used by XPath and XSLT.
///
/// This collection can be prepared before any XPath or XSLT processing begins.
///
/// Alternatively this collection can be added to incrementally during
/// processing using the `fn:doc` function for instance. Once a document under
/// a URL is present, it won't be changed.
#[derive(Debug)]
pub struct OwnedDocuments {
    pub(crate) xot: Xot,
    pub(crate) documents: RefCell<xee_interpreter::xml::Documents>,
}

#[derive(Debug)]
pub struct MutableDocuments<'a> {
    xot: &'a mut Xot,
    documents: &'a RefCell<xee_interpreter::xml::Documents>,
}

#[derive(Debug)]
pub struct RefDocuments<'a> {
    documents: &'a RefCell<xee_interpreter::xml::Documents>,
}

trait DocumentsAccess {
    fn get_node_by_handle(&self, handle: DocumentHandle) -> Option<xot::Node>;
    fn get_node_by_uri(&self, uri: &Uri) -> Option<xot::Node>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

trait DocumentsMut {
    /// Load a string as an XML document. Designate it with a URI.
    ///
    /// Something may go wrong during processing of the XML document; this is
    /// a [`xot::Error`].
    fn add_string(&mut self, uri: &Uri, xml: &str) -> Result<DocumentHandle, DocumentsError>;
}

impl OwnedDocuments {
    /// Create a new empty collection of documents.
    pub fn new() -> Self {
        Self {
            xot: Xot::new(),
            documents: RefCell::new(xee_interpreter::xml::Documents::new()),
        }
    }

    /// Load a string as an XML document. Designate it with a URI.
    ///
    /// Something may go wrong during processing of the XML document; this is
    /// a [`xot::Error`].
    pub fn add_string(&mut self, uri: &Uri, xml: &str) -> Result<DocumentHandle, DocumentsError> {
        self.documents
            .borrow_mut()
            .add_string(&mut self.xot, uri, xml)
    }
}

impl DocumentsMut for OwnedDocuments {
    fn add_string(&mut self, uri: &Uri, xml: &str) -> Result<DocumentHandle, DocumentsError> {
        self.add_string(uri, xml)
    }
}

impl<'a> MutableDocuments<'a> {
    pub fn new(xot: &'a mut Xot, documents: &'a RefCell<xee_interpreter::xml::Documents>) -> Self {
        Self { xot, documents }
    }

    pub fn add_string(&mut self, uri: &Uri, xml: &str) -> Result<DocumentHandle, DocumentsError> {
        self.documents.borrow_mut().add_string(self.xot, uri, xml)
    }
}

impl DocumentsMut for MutableDocuments<'_> {
    fn add_string(&mut self, uri: &Uri, xml: &str) -> Result<DocumentHandle, DocumentsError> {
        self.add_string(uri, xml)
    }
}

impl<'a> RefDocuments<'a> {
    pub fn new(documents: &'a RefCell<xee_interpreter::xml::Documents>) -> Self {
        Self { documents }
    }

    pub fn get_node_by_handle(&self, handle: DocumentHandle) -> Option<xot::Node> {
        self.documents.borrow().get_node_by_handle(handle)
    }

    pub fn get_node_by_uri(&self, uri: &Uri) -> Option<xot::Node> {
        self.documents.borrow().get_node_by_uri(uri)
    }

    pub fn len(&self) -> usize {
        self.documents.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.borrow().is_empty()
    }
}

impl<'a> DocumentsAccess for RefDocuments<'a> {
    fn get_node_by_handle(&self, handle: DocumentHandle) -> Option<xot::Node> {
        self.get_node_by_handle(handle)
    }

    fn get_node_by_uri(&self, uri: &Uri) -> Option<xot::Node> {
        self.get_node_by_uri(uri)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
