use ahash::AHashMap;
use std::borrow::Cow;
use std::fmt::Debug;

use xee_xpath_ast::ast;

use crate::error::Error;
use crate::sequence;
use crate::xml;

use super::static_context::StaticContext;

pub type Variables = AHashMap<ast::Name, sequence::Sequence>;

#[derive(Debug)]
pub struct DynamicContext<'a> {
    pub static_context: &'a StaticContext<'a>,
    pub documents: xml::Documents,
    pub(crate) variables: Cow<'a, Variables>,
    current_datetime: chrono::DateTime<chrono::offset::FixedOffset>,
}

impl<'a> DynamicContext<'a> {
    pub fn new(
        static_context: &'a StaticContext<'a>,
        documents: xml::Documents,
        variables: Cow<'a, Variables>,
    ) -> Self {
        Self {
            static_context,
            documents,
            variables,
            current_datetime: Self::create_current_datetime(),
        }
    }

    pub fn empty(static_context: &'a StaticContext<'a>) -> Self {
        let documents = xml::Documents::new();
        Self::new(static_context, documents, Cow::Owned(Variables::default()))
    }

    pub fn from_documents(
        static_context: &'a StaticContext<'a>,
        documents: xml::Documents,
    ) -> Self {
        Self::new(static_context, documents, Cow::Owned(Variables::default()))
    }

    pub fn from_variables(static_context: &'a StaticContext<'a>, variables: &'a Variables) -> Self {
        Self::new(
            static_context,
            xml::Documents::new(),
            Cow::Borrowed(variables),
        )
    }

    fn create_current_datetime() -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::offset::Local::now().into()
    }

    pub fn arguments(&self) -> Result<Vec<sequence::Sequence>, Error> {
        let mut arguments = Vec::new();
        for variable_name in &self.static_context.parser_context.variable_names {
            let items = self.variables.get(variable_name).ok_or(Error::XPDY0002)?;
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

    pub fn documents(&self) -> &xml::Documents {
        &self.documents
    }
}
