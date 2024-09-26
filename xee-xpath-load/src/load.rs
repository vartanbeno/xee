use anyhow::Result;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use xee_xpath_compiler::{context::StaticContext, sequence::Item, Namespaces};

use xee_xpath::{error::Result as XPathResult, Uri};
use xee_xpath::{DocumentHandle, Documents, Queries, Query, Session};

pub fn convert_string(_: &mut Session, item: &Item) -> XPathResult<String> {
    Ok(item.to_atomic()?.try_into()?)
}

pub fn convert_boolean(session: &mut Session, item: &Item) -> XPathResult<bool> {
    Ok(convert_string(session, item)? == "true")
}

pub trait ContextLoadable<C: ?Sized>: Sized {
    fn xpath_namespaces<'namespaces>() -> Namespaces<'namespaces>;

    fn load_with_context<'a>(
        queries: Queries<'a>,
        context: &'a C,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        Self: 'a;

    fn load_from_xml_with_context(xml: &str, context: &C) -> Result<Self> {
        let mut documents = Documents::new();
        // TODO: default document URI is just hardcoded
        let document_id = documents.add_string(&Uri::new("http://example.com"), xml)?;

        Self::load_from_node_with_context(documents, document_id, context)
    }

    fn load_from_node_with_context(
        documents: Documents,
        document_id: DocumentHandle,
        context: &C,
    ) -> Result<Self> {
        let static_context = StaticContext::from_namespaces(Self::xpath_namespaces());
        let queries = Queries::new(static_context);

        let (queries, query) = Self::load_with_context(queries, context)?;

        let mut session = queries.session(documents);

        Ok(query.execute(&mut session, document_id)?)
    }
}

pub trait Loadable: Sized {
    fn xpath_namespaces<'namespaces>() -> Namespaces<'namespaces>;

    fn load(queries: Queries) -> Result<(Queries, impl Query<Self>)>;

    fn load_from_xml(xml: &str) -> Result<Self> {
        Self::load_from_xml_with_context(xml, &())
    }

    fn load_from_node(documents: Documents, document_id: DocumentHandle) -> Result<Self> {
        Self::load_from_node_with_context(documents, document_id, &())
    }
}

impl<T: Loadable> ContextLoadable<()> for T {
    fn xpath_namespaces<'namespaces>() -> Namespaces<'namespaces> {
        T::xpath_namespaces()
    }

    fn load_with_context<'a>(
        queries: Queries<'a>,
        _context: &'a (),
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        T: 'a,
    {
        Self::load(queries)
    }
}

pub trait PathLoadable: ContextLoadable<Path> {
    fn load_from_file(path: &Path) -> Result<Self> {
        let xml_file = File::open(path)?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader.read_to_string(&mut xml)?;
        Self::load_from_xml_with_context(&xml, path)
    }
}

impl<T: ContextLoadable<Path>> PathLoadable for T {}
