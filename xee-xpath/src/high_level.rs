use std::{cell::RefCell, sync::atomic};

use crate::parse;

static XPATH_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
static DOCUMENTS_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

fn get_documents_id() -> usize {
    DOCUMENTS_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
}

fn get_xpath_id() -> usize {
    XPATH_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
}

#[derive(Debug)]
pub struct Documents {
    id: usize,
    xot: xot::Xot,
    document_uris: Vec<xee_interpreter::xml::Uri>,
    pub(crate) documents: RefCell<xee_interpreter::xml::Documents>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DocumentHandle {
    documents_id: usize,
    id: usize,
}

impl Documents {
    pub fn new() -> Self {
        Self {
            id: get_documents_id(),
            xot: xot::Xot::new(),
            document_uris: Vec::new(),
            documents: RefCell::new(xee_interpreter::xml::Documents::new()),
        }
    }

    pub fn load_string(&mut self, uri: &str, xml: &str) -> Result<DocumentHandle, xot::Error> {
        let id = self.document_uris.len();
        let uri = xee_interpreter::xml::Uri::new(uri);
        self.documents.borrow_mut().add(&mut self.xot, &uri, xml)?;
        self.document_uris.push(uri);
        Ok(DocumentHandle {
            documents_id: self.id,
            id,
        })
    }
}

#[derive(Debug)]
pub struct XPath<'namespace> {
    id: usize,
    static_context: xee_interpreter::context::StaticContext<'namespace>,
    xpath_programs: Vec<xee_interpreter::interpreter::Program>,
}

impl<'namespace> Default for XPath<'namespace> {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct XPathHandle {
    xpath_id: usize,
    id: usize,
}

impl<'namespace> XPath<'namespace> {
    pub fn new(default_element_namespace: &'namespace str) -> Self {
        let namespaces = xee_xpath_ast::Namespaces::new(
            xee_xpath_ast::Namespaces::default_namespaces(),
            default_element_namespace,
            xee_xpath_ast::FN_NAMESPACE,
        );
        let static_context = xee_interpreter::context::StaticContext::from_namespaces(namespaces);
        Self {
            id: get_xpath_id(),
            static_context,
            xpath_programs: Vec::new(),
        }
    }

    pub fn compile(
        &mut self,
        xpath: &str,
    ) -> Result<XPathHandle, xee_interpreter::error::SpannedError> {
        let id = self.xpath_programs.len();
        let program = parse(&self.static_context, xpath)?;
        self.xpath_programs.push(program);
        Ok(XPathHandle {
            xpath_id: self.id,
            id,
        })
    }
}

#[derive(Debug)]
pub struct Engine<'namespace> {
    xpath: &'namespace XPath<'namespace>,
    documents: Documents,
}

impl<'namespace> Engine<'namespace> {
    pub fn new(xpath: &'namespace XPath, documents: Documents) -> Self {
        Self { xpath, documents }
    }

    pub fn evaluate(
        &mut self,
        xpath_handle: XPathHandle,
        document_handle: DocumentHandle,
    ) -> Result<xee_interpreter::sequence::Sequence, xee_interpreter::error::SpannedError> {
        assert!(xpath_handle.xpath_id == self.xpath.id);
        let program = &self.xpath.xpath_programs[xpath_handle.id];
        assert!(document_handle.documents_id == self.documents.id);
        let document_uri = &self.documents.document_uris[document_handle.id];
        let root = {
            let borrowed_documents = self.documents.documents.borrow();
            let document = borrowed_documents.get(document_uri).unwrap();
            document.root()
        };
        let dynamic_context = xee_interpreter::context::DynamicContext::from_documents(
            &self.xpath.static_context,
            &self.documents.documents,
        );
        program
            .runnable(&dynamic_context)
            .many_xot_node(root, &mut self.documents.xot)
    }
}
