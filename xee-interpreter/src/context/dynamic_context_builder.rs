use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::{sequence, xml};

use super::{DynamicContext, StaticContext, Variables};

#[derive(Debug, Clone)]
pub struct DynamicContextBuilder<'a> {
    static_context: Rc<StaticContext<'a>>,
    context_item: Option<sequence::Item>,
    documents: Cow<'a, RefCell<xml::Documents>>,
    variables: Variables,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContextBuilder<'a> {
    /// Construct a new `DynamicContextBuilder` with the given `StaticContext`.
    pub fn new(static_context: Rc<StaticContext<'a>>) -> Self {
        Self {
            static_context,
            context_item: None,
            documents: Cow::Owned(RefCell::new(xml::Documents::new())),
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
    /// Give it owned documents and the [`DynamicContext`] will own them.
    pub fn owned_documents(&mut self, documents: xml::Documents) -> &mut Self {
        self.documents = Cow::Owned(RefCell::new(documents));
        self
    }

    /// Set the documents of the [`DynamicContext`].
    ///
    /// Give it a RefCell of documents and the [`DynamicContext`] will borrow them.
    pub fn ref_documents(&mut self, documents: &'a RefCell<xml::Documents>) -> &mut Self {
        self.documents = Cow::Borrowed(documents);
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
    pub fn build(&self) -> DynamicContext<'a> {
        DynamicContext::new(
            Rc::clone(&self.static_context),
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
        let builder = DynamicContextBuilder::new(Rc::new(static_context));
        let dynamic_context = builder.build();
        assert_eq!(dynamic_context.documents().borrow().len(), 0);
    }
}
