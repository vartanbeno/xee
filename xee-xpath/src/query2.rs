use std::sync::{atomic, Arc};

use anyhow::Result;

use xee_xpath_compiler::interpreter::Program;
use xee_xpath_compiler::parse;
use xee_xpath_compiler::sequence::Item;
use xee_xpath_compiler::{context::DynamicContext, context::StaticContext};
use xot::Xot;

use crate::{DocumentHandle, Documents};

static QUERIES_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

fn get_queries_id() -> usize {
    QUERIES_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
}

trait Itemable {
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

pub trait Convert<V>: Fn(&mut Session, &Item) -> Result<V> {}
impl<V, T> Convert<V> for T where T: Fn(&mut Session, &Item) -> Result<V> {}

// Recursion was very hard to get right. The trick is to use an intermediate
// struct.
// https://stackoverflow.com/questions/16946888/is-it-possible-to-make-a-recursive-closure-in-rust

// The dyn and dereference are unavoidable, as closures are not allowed
// to refer to themselves:
// https://github.com/rust-lang/rust/issues/46062
type RecurseFn<'s, V> = &'s dyn Fn(&mut Session, &Item, &Recurse<'s, V>) -> Result<V>;

pub struct Recurse<'s, V> {
    f: RecurseFn<'s, V>,
}

impl<'s, V> Recurse<'s, V> {
    pub fn new(f: RecurseFn<'s, V>) -> Self {
        Self { f }
    }
    pub fn execute(&self, session: &mut Session, item: &Item) -> Result<V> {
        (self.f)(session, item, self)
    }
}

pub trait ConvertRecurse<V>: Fn(&Session, &Item) -> Result<V> {}

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
    pub fn new(static_context: StaticContext<'namespaces>) -> Self {
        Self {
            id: get_queries_id(),
            xpath_programs: Vec::new(),
            static_context,
        }
    }

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

    pub fn one<V, F>(&mut self, s: &str, convert: F) -> Result<OneQuery<V, F>>
    where
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(OneQuery {
            id,
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn one_recurse(&mut self, s: &str) -> Result<OneRecurseQuery> {
        let id = self.register(s)?;
        Ok(OneRecurseQuery { id })
    }

    pub fn option<V, F>(&mut self, s: &str, convert: F) -> Result<OptionQuery<V, F>>
    where
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(OptionQuery {
            id,
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn option_recurse(&mut self, s: &str) -> Result<OptionRecurseQuery> {
        let id = self.register(s)?;
        Ok(OptionRecurseQuery { id })
    }

    pub fn many<V, F>(&mut self, s: &str, convert: F) -> Result<ManyQuery<V, F>>
    where
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(ManyQuery {
            id,
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn many_recurse(&mut self, s: &str) -> Result<ManyRecurseQuery> {
        let id = self.register(s)?;
        Ok(ManyRecurseQuery { id })
    }
}

#[derive(Debug)]
pub struct Session<'namespaces> {
    queries: &'namespaces Queries<'namespaces>,
    documents: Documents,
}

impl<'namespaces> Session<'namespaces> {
    pub fn new(queries: &'namespaces Queries<'namespaces>, documents: Documents) -> Self {
        Self { queries, documents }
    }

    fn dynamic_context(&self) -> DynamicContext {
        xee_interpreter::context::DynamicContext::from_documents(
            &self.queries.static_context,
            &self.documents.documents,
        )
    }

    fn query_program(&self, id: usize) -> &'namespaces Program {
        &self.queries.xpath_programs[id]
    }
}

pub trait Query<V> {
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V>;

    fn map<T>(
        self,
        f: impl Fn(V, &mut Session, &Item) -> Result<T> + Clone,
    ) -> MapQuery<V, T, Self, impl Fn(V, &mut Session, &Item) -> Result<T> + Clone>
    where
        Self: Sized,
    {
        MapQuery {
            query: self,
            f,
            v: std::marker::PhantomData,
            t: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapQuery<V, T, Q: Query<V> + Sized, F>
where
    F: Fn(V, &mut Session, &Item) -> Result<T> + Clone,
{
    query: Q,
    f: F,
    v: std::marker::PhantomData<V>,
    t: std::marker::PhantomData<T>,
}

impl<V, T, Q: Query<V> + Sized, F> Query<T> for MapQuery<V, T, Q, F>
where
    F: Fn(V, &mut Session, &Item) -> Result<T> + Clone,
{
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<T> {
        let v = self.query.execute(session, item)?;
        (self.f)(v, session, item)
    }
}

#[derive(Debug, Clone)]
pub struct OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    id: usize,
    convert: F,
    phantom: std::marker::PhantomData<V>,
}

impl<V, F> OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    pub fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V> {
        let program = session.query_program(self.id);
        let dynamic_context = xee_interpreter::context::DynamicContext::from_documents(
            &session.queries.static_context,
            &session.documents.documents,
        );
        let runnable = program.runnable(&dynamic_context);
        let item = item.to_item(session)?;
        let item = runnable.one(Some(&item), &mut session.documents.xot)?;
        (self.convert)(session, &item)
    }
}

impl<V, F> Query<V> for OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V> {
        Self::execute(self, session, item)
    }
}

#[derive(Debug, Clone)]
pub struct OneRecurseQuery {
    id: usize,
}

impl OneRecurseQuery {
    pub fn execute<V>(
        &self,
        session: &mut Session,
        item: &Item,
        recurse: &Recurse<V>,
    ) -> Result<V> {
        let program = session.query_program(self.id);
        let dynamic_context = xee_interpreter::context::DynamicContext::from_documents(
            &session.queries.static_context,
            &session.documents.documents,
        );
        let runnable = program.runnable(&dynamic_context);
        let item = runnable.one(Some(item), &mut session.documents.xot)?;
        recurse.execute(session, &item)
    }
}

#[derive(Debug, Clone)]
pub struct OptionQuery<V, F>
where
    F: Convert<V> + Copy,
{
    id: usize,
    convert: F,
    phantom: std::marker::PhantomData<V>,
}

impl<V, F> OptionQuery<V, F>
where
    F: Convert<V> + Copy,
{
    pub fn execute(&self, session: &mut Session, item: &Item) -> Result<Option<V>> {
        let program = session.query_program(self.id);
        let runnable = program.runnable(&session.dynamic_context);
        let item = runnable
            .option(Some(item), session.xot)
            .map_err(|e| e.error)?;
        if let Some(item) = item {
            match (self.convert)(session, &item) {
                Ok(value) => Ok(Some(value)),
                Err(query_error) => Err(query_error),
            }
        } else {
            Ok(None)
        }
    }
}

impl<V, F> Query<Option<V>> for OptionQuery<V, F>
where
    F: Convert<V> + Copy,
{
    fn execute(&self, session: &mut Session, item: &Item) -> Result<Option<V>> {
        Self::execute(self, session, item)
    }
}

#[derive(Debug, Clone)]
pub struct OptionRecurseQuery {
    id: usize,
}

impl OptionRecurseQuery {
    pub fn execute<V>(
        &self,
        session: &mut Session,
        item: &Item,
        recurse: &Recurse<V>,
    ) -> Result<Option<V>> {
        let program = session.query_program(self.id);
        let runnable = program.runnable(&session.dynamic_context);
        let item = runnable
            .option(Some(item), session.xot)
            .map_err(|e| e.error)?;
        if let Some(item) = item {
            Ok(Some(recurse.execute(session, &item)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManyQuery<V, F>
where
    F: Convert<V> + Copy,
{
    id: usize,
    convert: F,
    phantom: std::marker::PhantomData<V>,
}

impl<V, F> ManyQuery<V, F>
where
    F: Convert<V> + Copy,
{
    pub fn execute(&self, session: &mut Session, item: &Item) -> Result<Vec<V>> {
        let program = session.query_program(self.id);
        let runnable = program.runnable(&session.dynamic_context);
        let sequence = runnable
            .many(Some(item), session.xot)
            .map_err(|e| e.error)?;
        let mut values = Vec::with_capacity(sequence.len());
        for item in sequence.items()? {
            match (self.convert)(session, &item) {
                Ok(value) => values.push(value),
                Err(query_error) => return Err(query_error),
            }
        }
        Ok(values)
    }
}

impl<V, F> Query<Vec<V>> for ManyQuery<V, F>
where
    F: Convert<V> + Copy,
{
    fn execute(&self, session: &mut Session, item: &Item) -> Result<Vec<V>> {
        Self::execute(self, session, item)
    }
}

#[derive(Debug, Clone)]
pub struct ManyRecurseQuery {
    id: usize,
}

impl ManyRecurseQuery {
    pub fn execute<V>(
        &self,
        session: &mut Session,
        item: &Item,
        recurse: &Recurse<V>,
    ) -> Result<Vec<V>> {
        let program = session.query_program(self.id);
        let runnable = program.runnable(&session.dynamic_context);
        let sequence = runnable
            .many(Some(item), session.xot)
            .map_err(|e| e.error)?;
        let mut values = Vec::with_capacity(sequence.len());
        for item in sequence.items()? {
            values.push(recurse.execute(session, &item)?);
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use ibig::{ibig, IBig};
    use xot::Xot;

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

    #[test]
    fn test_nested_query() {
        let mut documents = Documents::new();
        let doc = documents
            .load_string("http://example.com", "<root><a>Alpha</a><a>Beta</a></root>")
            .unwrap();
        let mut queries = Queries::default();
        let string_query = queries
            .one("./string()", |_, item| {
                let s: String = item.to_atomic()?.try_into()?;
                Ok(s)
            })
            .unwrap();

        let q = queries
            .many("/root/a", |session, item| {
                string_query.execute(&mut session, item)
            })
            .unwrap();

        let mut session = queries.session(documents);
        let r = q.execute(&mut session, doc).unwrap();
        assert_eq!(r, "foo");
    }

    // #[test]
    // fn test_one_query_recurse() -> Result<()> {
    //     let static_context = StaticContext::default();
    //     let documents = RefCell::new(Documents::new());
    //     let dynamic_context = DynamicContext::from_documents(&static_context, &documents);
    //     let mut queries = Queries::new(&dynamic_context.static_context);
    //     #[derive(Debug, PartialEq, Eq)]
    //     enum Expr {
    //         AnyOf(Box<Expr>),
    //         Value(String),
    //         Empty,
    //     }

    //     let any_of_recurse = queries.option_recurse("any-of")?;
    //     let value_query = queries.option("value/string()", |_, item| {
    //         Ok(item.to_atomic()?.to_string()?)
    //     })?;

    //     let result_query = queries.one("doc/result", |session: &mut Session, item: &Item| {
    //         let f = |session: &mut Session, item: &Item, recurse: &Recurse<Expr>| {
    //             let any_of = any_of_recurse.execute(session, item, recurse)?;
    //             if let Some(any_of) = any_of {
    //                 return Ok(Expr::AnyOf(Box::new(any_of)));
    //             }
    //             if let Some(value) = value_query.execute(session, item)? {
    //                 return Ok(Expr::Value(value));
    //             }
    //             Ok(Expr::Empty)
    //         };
    //         let recurse = Recurse::new(&f);
    //         recurse.execute(session, item)
    //     })?;

    //     let mut xot = Xot::new();
    //     let xml = "<doc><result><any-of><value>A</value></any-of></result></doc>";
    //     let root = xot.parse(xml).unwrap();
    //     let xml2 = "<doc><result><value>A</value></result></doc>";
    //     let root2 = xot.parse(xml2).unwrap();

    //     let mut session = queries.session(&dynamic_context, &mut xot);
    //     let r = result_query.execute(&mut session, &Item::from(root))?;
    //     assert_eq!(r, Expr::AnyOf(Box::new(Expr::Value("A".to_string()))));

    //     let mut session = queries.session(&dynamic_context, &mut xot);
    //     let r = result_query.execute(&mut session, &Item::from(root2))?;
    //     assert_eq!(r, Expr::Value("A".to_string()));
    //     Ok(())
    // }

    // #[test]
    // fn test_map_query() {
    //     let static_context = StaticContext::default();
    //     let documents = RefCell::new(Documents::new());
    //     let dynamic_context = DynamicContext::from_documents(&static_context, &documents);
    //     let mut queries = Queries::new(&dynamic_context.static_context);
    //     let q = queries
    //         .one("1 + 2", |_, item| {
    //             let v: IBig = item.to_atomic()?.try_into()?;
    //             Ok(v)
    //         })
    //         .unwrap()
    //         .map(|v, _, _| Ok(v + ibig!(1)));

    //     let mut xot = Xot::new();

    //     let mut session = queries.session(&dynamic_context, &mut xot);
    //     let r = q.execute(&mut session, &1i64.into()).unwrap();
    //     assert_eq!(r, ibig!(4));
    // }

    // #[test]
    // fn test_map_query_clone() {
    //     let static_context = StaticContext::default();
    //     let documents = RefCell::new(Documents::new());
    //     let dynamic_context = DynamicContext::from_documents(&static_context, &documents);
    //     let mut queries = Queries::new(&dynamic_context.static_context);
    //     let q = queries
    //         .one("1 + 2", |_, item| {
    //             let v: IBig = item.to_atomic()?.try_into()?;
    //             Ok(v)
    //         })
    //         .unwrap()
    //         .map(|v, _, _| Ok(v + ibig!(1)));
    //     let q = q.clone();

    //     let mut xot = Xot::new();

    //     let mut session = queries.session(&dynamic_context, &mut xot);
    //     let r = q.execute(&mut session, &1i64.into()).unwrap();
    //     assert_eq!(r, ibig!(4));
    // }
}
