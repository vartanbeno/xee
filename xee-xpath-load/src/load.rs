use anyhow::Result;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use xee_xpath::{
    context::{DynamicContext, StaticContext},
    sequence::Item,
    Namespaces, Queries, Query, Session,
};
use xot::Xot;

pub fn convert_string(_: &mut Session, item: &Item) -> xee_xpath::error::Result<String> {
    item.to_atomic()?.try_into()
}

pub fn convert_boolean(session: &mut Session, item: &Item) -> xee_xpath::error::Result<bool> {
    Ok(convert_string(session, item)? == "true")
}

pub trait ContextLoadable<C: ?Sized>: Sized {
    fn query_with_context<'a>(
        queries: Queries<'a>,
        context: &'a C,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        Self: 'a;

    fn load_from_xml_with_context(
        xot: &mut Xot,
        namespaces: Namespaces,
        xml: &str,
        context: &C,
    ) -> Result<Self> {
        let root = xot.parse(xml)?;
        let document_element = xot.document_element(root)?;
        Self::load_with_context(xot, namespaces, document_element, context)
    }
    fn load_with_context(
        xot: &mut Xot,
        namespaces: Namespaces,
        node: xot::Node,
        context: &C,
    ) -> Result<Self> {
        let static_context = StaticContext::from_namespaces(namespaces);
        let queries = Queries::new(&static_context);
        let r = {
            let dynamic_context = DynamicContext::empty(&static_context);
            let (queries, query) = Self::query_with_context(queries, context)?;

            let mut session = queries.session(&dynamic_context, xot);
            // the query has a lifetime for the dynamic context, and a lifetime
            // for the static context
            query.execute(&mut session, &Item::from(node))?
        };
        Ok(r)
    }
}

pub trait Loadable: Sized {
    fn query(queries: Queries) -> Result<(Queries, impl Query<Self>)>;

    fn load_from_xml(xot: &mut Xot, namespaces: Namespaces, xml: &str) -> Result<Self> {
        Self::load_from_xml_with_context(xot, namespaces, xml, &())
    }

    fn load(xot: &mut Xot, namespaces: Namespaces, node: xot::Node) -> Result<Self> {
        Self::load_with_context(xot, namespaces, node, &())
    }
}

impl<T: Loadable> ContextLoadable<()> for T {
    fn query_with_context<'a>(
        queries: Queries<'a>,
        _context: &'a (),
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        T: 'a,
    {
        Self::query(queries)
    }
}

pub trait PathLoadable: ContextLoadable<Path> {
    fn load_from_file(xot: &mut Xot, namespaces: Namespaces, path: &Path) -> Result<Self> {
        let xml_file = File::open(path)?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader.read_to_string(&mut xml)?;
        Self::load_from_xml_with_context(xot, namespaces, &xml, path)
    }
}

impl<T: ContextLoadable<Path>> PathLoadable for T {}
