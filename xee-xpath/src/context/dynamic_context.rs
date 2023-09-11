use ahash::{HashMap, HashMapExt};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use xot::Xot;

use xee_xpath_ast::ast;

use crate::error::Error;
use crate::sequence;
use crate::xml;

use super::static_context::StaticContext;

pub struct DynamicContext<'a> {
    pub(crate) xot: &'a Xot,
    pub static_context: &'a StaticContext<'a>,
    pub(crate) documents: Cow<'a, xml::Documents>,
    pub(crate) variables: HashMap<ast::Name, Vec<sequence::Item>>,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
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
    // TODO: these constructor functions are ripe for refactoring.

    pub fn new(xot: &'a Xot, static_context: &'a StaticContext<'a>) -> Self {
        let documents = xml::Documents::new();
        Self {
            xot,
            static_context,
            documents: Cow::Owned(documents),
            variables: HashMap::new(),
            current_datetime: Self::create_current_datetime(),
        }
    }

    pub fn with_documents_and_variables(
        xot: &'a Xot,
        static_context: &'a StaticContext<'a>,
        documents: &'a xml::Documents,
        variables: &[(ast::Name, Vec<sequence::Item>)],
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Borrowed(documents),
            variables: variables
                .iter()
                .map(|(name, items)| (name.clone(), items.clone()))
                .collect(),
            current_datetime: Self::create_current_datetime(),
        }
    }

    pub fn with_documents(
        xot: &'a Xot,
        static_context: &'a StaticContext<'a>,
        documents: &'a xml::Documents,
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Borrowed(documents),
            variables: HashMap::new(),
            current_datetime: Self::create_current_datetime(),
        }
    }

    pub fn with_variables(
        xot: &'a Xot,
        static_context: &'a StaticContext<'a>,
        variables: &[(ast::Name, Vec<sequence::Item>)],
    ) -> Self {
        Self {
            xot,
            static_context,
            documents: Cow::Owned(xml::Documents::new()),
            variables: variables
                .iter()
                .map(|(name, items)| (name.clone(), items.clone()))
                .collect(),
            current_datetime: Self::create_current_datetime(),
        }
    }

    fn create_current_datetime() -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::offset::Local::now().into()
    }

    pub(crate) fn arguments(&self) -> Result<Vec<Vec<sequence::Item>>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.static_context.variables {
            let items = self
                .variables
                .get(variable_name)
                .ok_or(Error::ComponentAbsentInDynamicContext)?;
            arguments.push(items.clone());
        }
        Ok(arguments)
    }

    pub(crate) fn current_datetime(&self) -> chrono::DateTime<chrono::offset::FixedOffset> {
        self.current_datetime
    }

    pub(crate) fn implicit_timezone(&self) -> chrono::FixedOffset {
        self.current_datetime.timezone()
    }
}
