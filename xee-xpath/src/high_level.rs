// use std::{cell::RefCell, sync::atomic};

// use xee_xpath_compiler::parse;

// use crate::sequence::Sequence;
// use crate::Item;

// static XPATHS_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
// static DOCUMENTS_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

// fn get_documents_id() -> usize {
//     DOCUMENTS_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
// }

// fn get_xpaths_id() -> usize {
//     XPATHS_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
// }

// /// A collection of XML documents as can be used by XPath and XSLT.
// ///
// /// This collection can be prepared before any XPath or XSLT processing begins.
// ///
// /// Alternatively this collection can be added to incrementally during
// /// processing using the `fn:doc` function for instance. Once a document under
// /// a URL is present, it won't be changed.
// #[derive(Debug)]
// pub struct Documents {
//     pub(crate) id: usize,
//     pub(crate) xot: xot::Xot,
//     pub(crate) document_uris: Vec<xee_interpreter::xml::Uri>,
//     pub(crate) documents: RefCell<xee_interpreter::xml::Documents>,
// }

// /// A handle to a document.
// ///
// /// This is an identifier into a [`Documents`] collection. You can
// /// freely copy it.
// ///
// /// You can use it to evaluate XPath expressions on the document using
// /// the [`Engine`].
// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
// pub struct DocumentHandle {
//     pub(crate) documents_id: usize,
//     pub(crate) id: usize,
// }

// impl Documents {
//     /// Create a new empty collection of documents.
//     pub fn new() -> Self {
//         Self {
//             id: get_documents_id(),
//             xot: xot::Xot::new(),
//             document_uris: Vec::new(),
//             documents: RefCell::new(xee_interpreter::xml::Documents::new()),
//         }
//     }

//     /// Load a string as an XML document. Designate it with a URI.
//     ///
//     /// Something may go wrong during processing of the XML document; this is
//     /// a [`xot::Error`].
//     pub fn load_string(&mut self, uri: &str, xml: &str) -> Result<DocumentHandle, xot::Error> {
//         let id = self.document_uris.len();
//         let uri = xee_interpreter::xml::Uri::new(uri);
//         self.documents.borrow_mut().add(&mut self.xot, &uri, xml)?;
//         self.document_uris.push(uri);
//         Ok(DocumentHandle {
//             documents_id: self.id,
//             id,
//         })
//     }
// }

// /// A collection of compiled XPath expressions.
// #[derive(Debug)]
// pub struct XPaths<'namespace> {
//     id: usize,
//     static_context: xee_interpreter::context::StaticContext<'namespace>,
//     xpath_programs: Vec<xee_interpreter::interpreter::Program>,
// }

// impl<'namespace> Default for XPaths<'namespace> {
//     fn default() -> Self {
//         Self::new("")
//     }
// }

// /// A handle to a compiled XPath expression.
// ///
// /// This is an identifier into a [`XPaths`] collection. You can freely copy it.
// ///
// /// You can use it to evaluate the expression using the [`Engine`].
// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
// pub struct XPathHandle {
//     xpaths_id: usize,
//     id: usize,
// }

// impl<'namespace> XPaths<'namespace> {
//     /// Create a new collection of compiled XPath expressions.
//     ///
//     /// The default namespace to use for unprefixed XML element names in
//     /// expressions is passed in. Leave it empty or use the default
//     /// implementation if you do not want elements to be namespaced.
//     pub fn new(default_element_namespace: &'namespace str) -> Self {
//         let namespaces = xee_xpath_ast::Namespaces::new(
//             xee_xpath_ast::Namespaces::default_namespaces(),
//             default_element_namespace,
//             xee_xpath_ast::FN_NAMESPACE,
//         );
//         let static_context = xee_interpreter::context::StaticContext::from_namespaces(namespaces);
//         Self {
//             id: get_xpaths_id(),
//             static_context,
//             xpath_programs: Vec::new(),
//         }
//     }

//     /// Compile an XPath expression.
//     pub fn compile(
//         &mut self,
//         xpath: &str,
//     ) -> Result<XPathHandle, xee_interpreter::error::SpannedError> {
//         let id = self.xpath_programs.len();
//         let program = parse(&self.static_context, xpath)?;
//         self.xpath_programs.push(program);
//         Ok(XPathHandle {
//             xpaths_id: self.id,
//             id,
//         })
//     }
// }

// /// An engine that can execute XPath expressions against a collection of
// /// documents.
// #[derive(Debug)]
// pub struct Engine<'namespace> {
//     xpaths: &'namespace XPaths<'namespace>,
//     documents: Documents,
// }

// impl<'namespace> Engine<'namespace> {
//     /// Create a new engine. This consists of reference to XPath expressions
//     /// and a collection of documents that these XPath expressions can access.
//     pub fn new(xpaths: &'namespace XPaths, documents: Documents) -> Self {
//         Self { xpaths, documents }
//     }

//     /// Evaluate an XPath expression against a document.
//     ///
//     /// The handles indicate which XPath expression and which document to use.
//     /// If you use a handle for the wrong [`XPaths`] or [`Documents`] collection,
//     /// this is an error.
//     pub fn evaluate(
//         &mut self,
//         xpath_handle: XPathHandle,
//         document_handle: DocumentHandle,
//     ) -> Result<Sequence, xee_interpreter::error::SpannedError> {
//         // TODO: turn these into normal errors so that any application won't crash
//         assert!(xpath_handle.xpaths_id == self.xpaths.id);
//         let program = &self.xpaths.xpath_programs[xpath_handle.id];
//         assert!(document_handle.documents_id == self.documents.id);
//         let document_uri = &self.documents.document_uris[document_handle.id];
//         let root = {
//             let borrowed_documents = self.documents.documents.borrow();
//             let document = borrowed_documents.get(document_uri).unwrap();
//             document.root()
//         };
//         let dynamic_context = xee_interpreter::context::DynamicContext::from_documents(
//             &self.xpaths.static_context,
//             self.documents.documents,
//         );
//         Ok(Sequence::new(
//             program
//                 .runnable(&dynamic_context)
//                 .many_xot_node(root, &mut self.documents.xot)?,
//         ))
//     }
// }
