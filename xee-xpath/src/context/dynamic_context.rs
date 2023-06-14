use ahash::{HashMap, HashMapExt};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use xot::Xot;

use xee_xpath_ast::ast;

use crate::error::Error;
use crate::output;
use crate::stack;
use crate::xml;

use super::static_context::StaticContext;

pub struct DynamicContext<'a> {
    pub(crate) xot: &'a Xot,
    pub(crate) static_context: &'a StaticContext<'a>,
    pub(crate) documents: Cow<'a, xml::Documents>,
    pub(crate) variables: HashMap<ast::Name, Vec<stack::StackItem>>,
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
        let documents = xml::Documents::new();
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
        documents: &'a xml::Documents,
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
        variables: &[(ast::Name, Vec<output::OutputItem>)],
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Owned(xml::Documents::new()),
            variables: variables
                .iter()
                .map(|(name, items)| (name.clone(), items.iter().map(|item| item.into()).collect()))
                .collect(),
        }
    }

    pub(crate) fn arguments(&self) -> Result<Vec<Vec<stack::StackItem>>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.static_context.variables {
            let items = self.variables.get(variable_name).ok_or(Error::XPDY0002A)?;
            arguments.push(items.clone());
        }
        Ok(arguments)
    }
}
