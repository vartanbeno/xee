use std::{cell::RefCell, ops::Deref, rc::Rc};

use ahash::{HashMap, HashMapExt};

use crate::{interpreter, sequence, xml};

use super::{DynamicContext, Variables};

/// A builder for constructing a [`DynamicContext`].
///
/// This needs to be supplied a [`StaticContext`] (or a reference to one) in
/// order to construct it.
///
/// You can supply a context item, documents, variables and the like in order
/// to construct a dynamic context used to execute an XPath instruction.
#[derive(Debug, Clone)]
pub struct DynamicContextBuilder<'a> {
    program: &'a interpreter::Program,
    context_item: Option<sequence::Item>,
    documents: DocumentsRef,
    variables: Variables,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
    default_collection: Option<sequence::Sequence>,
    collections: HashMap<String, sequence::Sequence>,
}

#[derive(Debug, Clone)]
pub struct DocumentsRef(Rc<RefCell<xml::Documents>>);

impl Deref for DocumentsRef {
    type Target = RefCell<xml::Documents>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<xml::Documents> for DocumentsRef {
    fn from(documents: xml::Documents) -> Self {
        Self(Rc::new(RefCell::new(documents)))
    }
}

impl DocumentsRef {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(xml::Documents::new())))
    }
}

impl Default for DocumentsRef {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DynamicContextBuilder<'a> {
    /// Construct a new `DynamicContextBuilder` with the given `StaticContext`.
    pub(crate) fn new(program: &'a interpreter::Program) -> Self {
        Self {
            program,
            context_item: None,
            documents: DocumentsRef::new(),
            variables: Variables::new(),
            current_datetime: chrono::offset::Local::now().into(),
            default_collection: None,
            collections: HashMap::new(),
        }
    }

    /// Set the context item of the [`DynamicContext`].
    ///
    /// Without this, the [`DynamicContext`] will have no context item.
    pub fn context_item(&mut self, context_item: sequence::Item) -> &mut Self {
        self.context_item = Some(context_item);
        self
    }

    /// Set a node as the context item of the [`DynamicContext`].
    pub fn context_node(&mut self, node: xot::Node) -> &mut Self {
        self.context_item(sequence::Item::Node(node));
        self
    }

    /// Set the documents of the [`DynamicContext`].
    ///
    /// You can give it either owned documents or a [`DocumentsRef`].
    pub fn documents(&mut self, documents: impl Into<DocumentsRef>) -> &mut Self {
        self.documents = documents.into();
        self
    }

    /// Set the variables of the [`DynamicContext`].
    ///
    /// Without this, the [`DynamicContext`] will have no variables.
    pub fn variables(&mut self, variables: Variables) -> &mut Self {
        self.variables = variables;
        self
    }

    /// Set the current datetime of the [`DynamicContext`].
    ///
    /// Without this, the [`DynamicContext`] will have the current datetime.
    pub fn current_datetime(
        &mut self,
        current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
    ) -> &mut Self {
        self.current_datetime = current_datetime;
        self
    }

    /// The the default collection
    pub fn default_collection(&mut self, sequence: sequence::Sequence) -> &mut Self {
        self.default_collection = Some(sequence);
        self
    }

    /// Set a collection
    pub fn collection(&mut self, uri: String, sequence: sequence::Sequence) -> &mut Self {
        self.collections.insert(uri, sequence);
        self
    }

    /// Build the `DynamicContext`.
    pub fn build(&self) -> DynamicContext {
        DynamicContext::new(
            self.program,
            self.context_item.clone(),
            self.documents.clone(),
            self.variables.clone(),
            self.current_datetime,
            self.default_collection.clone(),
            self.collections.clone(),
        )
    }
}
