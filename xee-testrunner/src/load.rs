use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use xee_name::Namespaces;
use xee_xpath::{
    context::{DynamicContext, StaticContext},
    sequence::Item,
    Queries, Query, Session,
};
use xot::Xot;

use crate::error::Result;

pub(crate) const XPATH_NS: &str = "http://www.w3.org/2010/09/qt-fots-catalog";

pub(crate) fn convert_string(_: &mut Session, item: &Item) -> xee_xpath::error::Result<String> {
    item.to_atomic()?.try_into()
}

pub(crate) fn convert_boolean(
    session: &mut Session,
    item: &Item,
) -> xee_xpath::error::Result<bool> {
    Ok(convert_string(session, item)? == "true")
}

pub(crate) trait ContextLoadable<C: ?Sized>: Sized {
    fn query_with_context<'a>(
        queries: Queries<'a>,
        context: &'a C,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        Self: 'a;

    fn load_from_xml_with_context(xot: &mut Xot, xml: &str, ns: &str, context: &C) -> Result<Self> {
        let root = xot.parse(xml)?;
        let document_element = xot.document_element(root)?;
        Self::load_with_context(xot, document_element, ns, context)
    }
    fn load_with_context(xot: &mut Xot, node: xot::Node, ns: &str, context: &C) -> Result<Self> {
        let namespaces = Namespaces::new(
            Namespaces::default_namespaces(),
            ns,
            Namespaces::FN_NAMESPACE,
        );
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

pub(crate) trait Loadable: Sized {
    fn query(queries: Queries) -> Result<(Queries, impl Query<Self>)>;

    fn load_from_xml(xot: &mut Xot, xml: &str, ns: &str) -> Result<Self> {
        Self::load_from_xml_with_context(xot, xml, ns, &())
    }

    fn load(xot: &mut Xot, node: xot::Node, ns: &str) -> Result<Self> {
        Self::load_with_context(xot, node, ns, &())
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

pub(crate) trait PathLoadable: ContextLoadable<Path> {
    fn load_from_file(xot: &mut Xot, ns: &str, path: &Path) -> Result<Self> {
        let xml_file = File::open(path)?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader.read_to_string(&mut xml)?;
        Self::load_from_xml_with_context(xot, &xml, ns, path)
    }
}

impl<T: ContextLoadable<Path>> PathLoadable for T {}
