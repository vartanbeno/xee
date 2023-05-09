use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use xot::Xot;

use crate::document::Documents;
use crate::static_context::StaticContext;

pub struct DynamicContext<'a> {
    pub(crate) xot: &'a Xot,
    pub(crate) static_context: StaticContext<'a>,
    pub(crate) documents: Cow<'a, Documents>,
}

impl<'a> Debug for DynamicContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("static_context", &self.static_context)
            .field("documents", &self.documents)
            .finish()
    }
}

impl<'a> DynamicContext<'a> {
    pub fn new(xot: &'a Xot, static_context: StaticContext<'a>) -> Self {
        let documents = Documents::new();
        Self {
            xot,
            static_context,
            documents: Cow::Owned(documents),
        }
    }

    pub(crate) fn with_documents(
        xot: &'a Xot,
        static_context: StaticContext<'a>,
        documents: &'a Documents,
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Borrowed(documents),
        }
    }
}
