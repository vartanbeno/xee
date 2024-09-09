use anyhow::Result;
use xee_interpreter::{context::DynamicContext, sequence::Item};
use xee_xpath_compiler::parse;

use std::sync::atomic;

use crate::{DocumentHandle, Documents};

static QUERIES_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

fn get_queries_id() -> usize {
    QUERIES_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
}

/// A query that can be executed against an [`Itemable`]
///
/// It gives back a result of type `V`
pub trait Query<V, F>
where
    F: Convert<V> + Copy,
{
    /// Excute the query against an itemable
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V>;
}

/// This is the core conversion function that can be used to turn
/// an item that results from an XPath query into something useful
/// in Rust.
///
/// Given a [`Session`] and an [`Item`], convert the item to a value of type `V`
pub trait Convert<V>: Fn(&mut Session, &Item) -> Result<V> {}
impl<V, T> Convert<V> for T where T: Fn(&mut Session, &Item) -> Result<V> {}

/// Something that can be converted into an [`Item`] using a [`Session`]
///
/// This can be an item, but also a [`DocumentHandle`]
pub trait Itemable {
    fn to_item(&self, session: &Session) -> Result<Item>;
}

impl Itemable for xot::Node {
    fn to_item(&self, _session: &Session) -> Result<Item> {
        Ok(Item::Node(self.clone()))
    }
}

impl Itemable for DocumentHandle {
    fn to_item(&self, session: &Session) -> Result<Item> {
        assert!(self.documents_id == session.documents.id);
        let document_uri = &session.documents.document_uris[self.id];
        let borrowed_documents = session.documents.documents.borrow();
        let document = borrowed_documents.get(document_uri).unwrap();
        Ok(Item::Node(document.root()))
    }
}

/// This is a query that expects a sequence that is one item long.
///
/// If it's empty or has more than one item, an error is returned.
#[derive(Debug, Clone, Copy)]
pub struct OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    queries_id: usize,
    id: usize,
    convert: F,
    phantom: std::marker::PhantomData<V>,
}

impl<V, F> Query<V, F> for OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V> {
        assert_eq!(self.queries_id, session.queries.id);
        let program = &session.queries.xpath_programs[self.id];
        let dynamic_context = DynamicContext::from_documents(
            &session.queries.static_context,
            &session.documents.documents,
        );
        let runnable = program.runnable(&dynamic_context);
        let item = item.to_item(session)?;
        let item = runnable.one(Some(&item), &mut session.documents.xot)?;
        (self.convert)(session, &item)
    }
}

/// A collection of XPath queries
///
/// You can register xpath expressions with conversion functions
/// to turn the results into Rust values.
#[derive(Debug)]
pub struct Queries<'namespaces> {
    id: usize,
    xpath_programs: Vec<xee_interpreter::interpreter::Program>,
    static_context: xee_interpreter::context::StaticContext<'namespaces>,
}

impl Default for Queries<'_> {
    fn default() -> Self {
        let default_element_namespace = "";
        let namespaces = xee_xpath_ast::Namespaces::new(
            xee_xpath_ast::Namespaces::default_namespaces(),
            default_element_namespace,
            xee_xpath_ast::FN_NAMESPACE,
        );
        let static_context = xee_interpreter::context::StaticContext::from_namespaces(namespaces);
        Self {
            id: get_queries_id(),
            xpath_programs: Vec::new(),
            static_context,
        }
    }
}

impl<'namespaces> Queries<'namespaces> {
    /// Construct a new collection of queries
    pub fn new(static_context: xee_interpreter::context::StaticContext<'namespaces>) -> Self {
        Self {
            id: get_queries_id(),
            xpath_programs: Vec::new(),
            static_context,
        }
    }

    /// Construct a [`Session`]` with a collection of documents
    ///
    /// A session is a context in which queries can be executed
    pub fn session(&self, documents: Documents) -> Session {
        Session::new(self, documents)
    }

    // pub fn session<'d>(
    //     &'d self,
    //     dynamic_context: &'d DynamicContext<'d>,
    //     xot: &'d mut Xot,
    // ) -> Session {
    //     Session::new(dynamic_context, self, xot)
    // }

    fn register(&mut self, s: &str) -> Result<usize> {
        let program = parse(&self.static_context, s).map_err(|e| e.error)?;
        let id = self.xpath_programs.len();
        self.xpath_programs.push(program);
        Ok(id)
    }

    /// Construct a query that expects a single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    pub fn one<V, F>(&mut self, s: &str, convert: F) -> Result<OneQuery<V, F>>
    where
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(OneQuery {
            queries_id: self.id,
            id,
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    // pub fn one_recurse(&mut self, s: &str) -> Result<OneRecurseQuery> {
    //     let id = self.register(s)?;
    //     Ok(OneRecurseQuery { id })
    // }

    // pub fn option<V, F>(&mut self, s: &str, convert: F) -> Result<OptionQuery<V, F>>
    // where
    //     F: Convert<V> + Copy,
    // {
    //     let id = self.register(s)?;
    //     Ok(OptionQuery {
    //         id,
    //         convert,
    //         phantom: std::marker::PhantomData,
    //     })
    // }

    // pub fn option_recurse(&mut self, s: &str) -> Result<OptionRecurseQuery> {
    //     let id = self.register(s)?;
    //     Ok(OptionRecurseQuery { id })
    // }

    // pub fn many<V, F>(&mut self, s: &str, convert: F) -> Result<ManyQuery<V, F>>
    // where
    //     F: Convert<V> + Copy,
    // {
    //     let id = self.register(s)?;
    //     Ok(ManyQuery {
    //         id,
    //         convert,
    //         phantom: std::marker::PhantomData,
    //     })
    // }

    // pub fn many_recurse(&mut self, s: &str) -> Result<ManyRecurseQuery> {
    //     let id = self.register(s)?;
    //     Ok(ManyRecurseQuery { id })queries
}

/// A session in which queries can be executed
#[derive(Debug)]
pub struct Session<'namespaces> {
    queries: &'namespaces Queries<'namespaces>,
    documents: Documents,
}

impl<'namespaces> Session<'namespaces> {
    fn new(queries: &'namespaces Queries<'namespaces>, documents: Documents) -> Self {
        Self { queries, documents }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_one_query() {
        let mut documents = Documents::new();
        let doc = documents
            .load_string("http://example.com", "<root>foo</root>")
            .unwrap();

        let mut queries = Queries::default();
        let q = queries
            .one("/root/string()", |_, item| {
                let s: String = item.to_atomic()?.try_into()?;
                Ok(s)
            })
            .unwrap();

        let mut session = queries.session(documents);
        let r = q.execute(&mut session, doc).unwrap();
        assert_eq!(r, "foo");
    }
}
