use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use xot::Xot;

use crate::document::Documents;
use crate::static_context::StaticContext;

pub(crate) struct Context<'a> {
    pub(crate) xot: &'a Xot,
    pub(crate) static_context: StaticContext,
    pub(crate) documents: Cow<'a, Documents>,
}

impl<'a> Debug for Context<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("static_context", &self.static_context)
            .field("documents", &self.documents)
            .finish()
    }
}

impl<'a> Context<'a> {
    pub(crate) fn new(xot: &'a Xot) -> Self {
        let documents = Documents::new();
        Self {
            xot,
            static_context: StaticContext::new(),
            documents: Cow::Owned(documents),
        }
    }

    pub(crate) fn with_documents(xot: &'a Xot, documents: &'a Documents) -> Self {
        Self {
            xot,
            static_context: StaticContext::new(),
            documents: Cow::Borrowed(documents),
        }
    }
}
