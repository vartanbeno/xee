use std::{cell::RefCell, sync::atomic};

static DOCUMENTS_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

fn get_documents_id() -> usize {
    DOCUMENTS_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
}

/// A collection of XML documents as can be used by XPath and XSLT.
///
/// This collection can be prepared before any XPath or XSLT processing begins.
///
/// Alternatively this collection can be added to incrementally during
/// processing using the `fn:doc` function for instance. Once a document under
/// a URL is present, it won't be changed.
#[derive(Debug)]
pub struct Documents {
    pub(crate) inner: InnerDocuments,
    pub(crate) documents: RefCell<xee_interpreter::xml::Documents>,
}

#[derive(Debug)]
pub(crate) struct InnerDocuments {
    pub(crate) id: usize,
    pub(crate) xot: xot::Xot,
    pub(crate) document_uris: Vec<xee_interpreter::xml::Uri>,
}

/// A handle to a document.
///
/// This is an identifier into a [`Documents`] collection. You can
/// freely copy it.
///
/// You can use it to evaluate XPath expressions on the document using
/// [`Queries`](`crate::Queries`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DocumentHandle {
    pub(crate) documents_id: usize,
    pub(crate) id: usize,
}

impl Documents {
    /// Create a new empty collection of documents.
    pub fn new() -> Self {
        Self {
            inner: InnerDocuments {
                id: get_documents_id(),
                xot: xot::Xot::new(),
                document_uris: Vec::new(),
            },
            documents: RefCell::new(xee_interpreter::xml::Documents::new()),
        }
    }

    /// Load a string as an XML document. Designate it with a URI.
    ///
    /// Something may go wrong during processing of the XML document; this is
    /// a [`xot::Error`].
    pub fn load_string(&mut self, uri: &str, xml: &str) -> Result<DocumentHandle, xot::Error> {
        let id = self.inner.document_uris.len();
        let uri = xee_interpreter::xml::Uri::new(uri);
        self.documents
            .borrow_mut()
            .add(&mut self.inner.xot, &uri, xml)?;
        self.inner.document_uris.push(uri);
        Ok(DocumentHandle {
            documents_id: self.inner.id,
            id,
        })
    }
}
