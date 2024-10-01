use std::cell::RefCell;

use xee_interpreter::xml::{DocumentHandle, DocumentsError, Uri};
use xot::Xot;

/// A collection of XML documents as can be used by XPath and XSLT.
///
/// This collection can be prepared before any XPath or XSLT processing begins.
///
/// Alternatively this collection can be added to incrementally during
/// processing using the `fn:doc` function for instance. Once a document under
/// a URL is present, it won't be changed.
#[derive(Debug)]
pub struct Documents {
    pub(crate) xot: Xot,
    pub(crate) documents: RefCell<xee_interpreter::xml::Documents>,
}

impl Documents {
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
