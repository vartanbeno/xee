use ahash::{HashMap, HashMapExt};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use xot::Xot;

use xee_xpath_ast::ast;

use crate::data::Value;
use crate::document::Documents;
use crate::error::Error;

use super::static_context::StaticContext;

pub struct DynamicContext<'a> {
    pub(crate) xot: &'a Xot,
    pub(crate) static_context: &'a StaticContext<'a>,
    pub(crate) documents: Cow<'a, Documents>,
    pub(crate) variables: HashMap<ast::Name, Value>,
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
    pub fn new(xot: &'a Xot, static_context: &'a StaticContext<'a>) -> Self {
        let documents = Documents::new();
        Self {
            xot,
            static_context,
            documents: Cow::Owned(documents),
            variables: HashMap::new(),
        }
    }

    pub(crate) fn with_documents(
        xot: &'a Xot,
        static_context: &'a StaticContext<'a>,
        documents: &'a Documents,
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Borrowed(documents),
            variables: HashMap::new(),
        }
    }

    pub fn with_variables(
        xot: &'a Xot,
        static_context: &'a StaticContext<'a>,
        variables: &[(ast::Name, Value)],
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Owned(Documents::new()),
            variables: variables.iter().cloned().collect(),
        }
    }

    pub(crate) fn arguments(&self) -> Result<Vec<Value>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.static_context.variables {
            let value = self.variables.get(variable_name).ok_or(Error::XPDY0002A)?;
            arguments.push(value.clone());
        }
        Ok(arguments)
    }
}
