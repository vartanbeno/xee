use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{sequence, xml};

use super::{DynamicContext, StaticContext, Variables};

/// A builder for constructing a [`DynamicContext`].
///
/// This needs to be supplied a [`StaticContext`] (or a reference to one) in
/// order to construct it.
///
/// You can supply a context item, documents, variables and the like in order
/// to construct a dynamic context used to execute an XPath instruction.
#[derive(Debug, Clone)]
pub struct DynamicContextBuilder {
    static_context: StaticContextRef,
    context_item: Option<sequence::Item>,
    documents: DocumentsRef,
    variables: Variables,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

#[derive(Debug, Clone)]
pub struct StaticContextRef(Rc<StaticContext>);

impl From<StaticContext> for StaticContextRef {
    fn from(static_context: StaticContext) -> Self {
        Self(Rc::new(static_context))
    }
}

impl Deref for StaticContextRef {
    type Target = StaticContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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

impl DynamicContextBuilder {
    /// Construct a new `DynamicContextBuilder` with the given `StaticContext`.
    pub fn new(static_context: impl Into<StaticContextRef>) -> Self {
        Self {
            static_context: static_context.into(),
            context_item: None,
            documents: DocumentsRef::new(),
            variables: Variables::new(),
            current_datetime: chrono::offset::Local::now().into(),
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

    /// Build the `DynamicContext`.
    pub fn build(&self) -> DynamicContext {
        DynamicContext::new(
            self.static_context.clone(),
            self.context_item.clone(),
            self.documents.clone(),
            self.variables.clone(),
            self.current_datetime,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_context_builder() {
        let static_context = StaticContext::default();
        let builder = DynamicContextBuilder::new(static_context);
        let dynamic_context = builder.build();
        assert_eq!(dynamic_context.documents().borrow().len(), 0);
    }
}
