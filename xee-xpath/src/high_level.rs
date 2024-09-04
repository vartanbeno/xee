use std::cell::RefCell;

use crate::parse;

#[derive(Debug)]
pub struct Documents {
    xot: xot::Xot,
    document_uris: Vec<xee_interpreter::xml::Uri>,
    pub(crate) documents: RefCell<xee_interpreter::xml::Documents>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DocumentHandle(usize);

impl Documents {
    pub fn new() -> Self {
        Self {
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
        Ok(DocumentHandle(id))
    }
}

#[derive(Debug)]
pub struct XPath<'namespace> {
    static_context: xee_interpreter::context::StaticContext<'namespace>,
    xpath_programs: Vec<xee_interpreter::interpreter::Program>,
}

impl<'namespace> Default for XPath<'namespace> {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct XPathHandle(usize);

impl<'namespace> XPath<'namespace> {
    pub fn new(default_element_namespace: &'namespace str) -> Self {
        let namespaces = xee_xpath_ast::Namespaces::new(
            xee_xpath_ast::Namespaces::default_namespaces(),
            default_element_namespace,
            xee_xpath_ast::FN_NAMESPACE,
        );
        let static_context = xee_interpreter::context::StaticContext::from_namespaces(namespaces);
        Self {
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
        Ok(XPathHandle(id))
    }
}

#[derive(Debug)]
pub struct Engine<'namespace> {
    xpath: &'namespace XPath<'namespace>,
}

impl<'namespace> Engine<'namespace> {
    pub fn new(xpath: &'namespace XPath) -> Self {
        Self { xpath: xpath }
    }

    pub fn evaluate(
        &mut self,
        xpath_handle: XPathHandle,
        // TODO: don't want to pass in new creation every time
        mut documents: Documents,
        document_handle: DocumentHandle,
    ) -> Result<xee_interpreter::sequence::Sequence, xee_interpreter::error::SpannedError> {
        let program = &self.xpath.xpath_programs[xpath_handle.0];
        // TODO what if documents does not have the document_handle because
        // we mixed them up?
        let document_uri = &documents.document_uris[document_handle.0];
        let borrowed_documents = documents.documents.borrow();
        let document = borrowed_documents.get(document_uri).unwrap();
        let root = document.root();
        let dynamic_context = xee_interpreter::context::DynamicContext::from_documents(
            &self.xpath.static_context,
            &documents.documents,
        );
        program
            .runnable(&dynamic_context)
            .many_xot_node(root, &mut documents.xot)
    }
}
