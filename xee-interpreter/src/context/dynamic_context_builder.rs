use std::{borrow::Cow, cell::RefCell};

use crate::xml;

use super::{static_context, DynamicContext, StaticContext, Variables};

#[derive(Debug, Clone)]
pub struct DynamicContextBuilder<'a> {
    static_context: &'a StaticContext<'a>,
    documents: Cow<'a, RefCell<xml::Documents>>,
    variables: Variables,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContextBuilder<'a> {
    /// Construct a new `DynamicContextBuilder` with the given `StaticContext`.
    pub fn new(static_context: &'a StaticContext<'a>) -> Self {
        Self {
            static_context,
            documents: Cow::Owned(RefCell::new(xml::Documents::new())),
            variables: Variables::new(),
            current_datetime: chrono::offset::Local::now().into(),
        }
    }

    /// Set the documents of the `DynamicContext`.
    ///
    /// Give it owned documents and the `DynamicContext` will own them.
    pub fn owned_documents(&mut self, documents: xml::Documents) -> &mut Self {
        self.documents = Cow::Owned(RefCell::new(documents));
        self
    }

    /// Set the documents of the `DynamicContext`.
    ///
    /// Give it a RefCell of documents and the `DynamicContext` will borrow them.
    pub fn ref_documents(&mut self, documents: &'a RefCell<xml::Documents>) -> &mut Self {
        self.documents = Cow::Borrowed(documents);
        self
    }

    /// Set the variables of the `DynamicContext`.
    ///
    /// Without this, the `DynamicContext` will have no variables.
    pub fn variables(&mut self, variables: Variables) -> &mut Self {
        self.variables = variables;
        self
    }

    /// Set the current datetime of the `DynamicContext`.
    ///
    /// Without this, the `DynamicContext` will have the current datetime.
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
            self.static_context,
            self.documents.clone(),
            self.variables.clone(),
            self.current_datetime,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::Documents;

    #[test]
    fn test_dynamic_context_builder() {
        let static_context = static_context::StaticContext::default();
        let mut builder = DynamicContextBuilder::new(&static_context);
        let dynamic_context = builder.build();
        assert_eq!(dynamic_context.documents().borrow().len(), 0);
    }
}
