use anyhow::Result;
use std::{
    borrow::BorrowMut,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use xee_xpath::{
    context::{DynamicContext, StaticContext},
    sequence::Item,
    xml::Uri,
    Namespaces,
};
use xot::Xot;

use crate::{Queries, Query, Session};

pub fn convert_string(_: &mut Session, item: &Item) -> Result<String> {
    Ok(item.to_atomic()?.try_into()?)
}

pub fn convert_boolean(session: &mut Session, item: &Item) -> Result<bool> {
    Ok(convert_string(session, item)? == "true")
}

pub trait ContextLoadable<C: ?Sized>: Sized {
    fn load_with_context<'a>(
        queries: Queries<'a>,
        context: &'a C,
    ) -> Result<(Queries<'a>, impl Query<Self> + 'a)>
    where
        Self: 'a;

    fn load_from_xml_with_context(
        xot: &mut Xot,
        dynamic_context: &mut DynamicContext,
        xml: &str,
        context: &C,
    ) -> Result<Self> {
        let root = xot.parse(xml)?;
        let document_element = xot.document_element(root)?;
        // TODO: Uri is a hack
        dynamic_context
            .documents
            .add_root(xot, &Uri::new("http://example.com"), root);
        Self::load_from_node_with_context(xot, dynamic_context, document_element, context)
    }

    fn load_from_node_with_context(
        xot: &mut Xot,
        dynamic_context: &mut DynamicContext,
        node: xot::Node,
        context: &C,
    ) -> Result<Self> {
        // let static_context = StaticContext::from_namespaces(namespaces);
        let queries = Queries::new(&dynamic_context.static_context);
        // let r = {
        //     let mut dynamic_context = DynamicContext::empty(&static_context);
        //     // TODO: this set up isn't proper yet, should really be a proper path,
        //     // but the main goal is to properly set up annotations
        //     let uri = Uri::new("http://example.com");
        //     dynamic_context.documents.add_root(xot, &uri, node);
        let (queries, query) = Self::load_with_context(queries, context)?;

        let mut session = queries.session(dynamic_context, xot);
        // the query has a lifetime for the dynamic context, and a lifetime
        // for the static context
        query.execute(&mut session, &Item::from(node))
    }
}

pub trait Loadable: Sized {
    fn load(queries: Queries) -> Result<(Queries, impl Query<Self>)>;

    fn load_from_xml(
        xot: &mut Xot,
        dynamic_context: &mut DynamicContext,
        xml: &str,
    ) -> Result<Self> {
        Self::load_from_xml_with_context(xot, dynamic_context, xml, &())
    }

    fn load_from_node(
        xot: &mut Xot,
        dynamic_context: &mut DynamicContext,
        node: xot::Node,
    ) -> Result<Self> {
        Self::load_from_node_with_context(xot, dynamic_context, node, &())
    }
}

impl<T: Loadable> ContextLoadable<()> for T {
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
    fn load_from_file(
        xot: &mut Xot,
        dynamic_context: &mut DynamicContext,
        path: &Path,
    ) -> Result<Self> {
        let xml_file = File::open(path)?;
        let mut buf_reader = BufReader::new(xml_file);
        let mut xml = String::new();
        buf_reader.read_to_string(&mut xml)?;
        Self::load_from_xml_with_context(xot, dynamic_context, &xml, path)
    }
}

impl<T: ContextLoadable<Path>> PathLoadable for T {}
