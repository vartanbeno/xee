use xee_interpreter::error::SpannedResult as Result;
use xee_interpreter::occurrence::Occurrence;
use xee_interpreter::sequence::{self, Item};

use std::sync::atomic;

use crate::session::Session;
use crate::Itemable;

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
    fn new(f: RecurseFn<'s, V>) -> Self {
        Self { f }
    }
    pub fn execute(&self, session: &mut Session, item: &Item) -> Result<V> {
        (self.f)(session, item, self)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QueryId {
    queries_id: usize,
    id: usize,
}

impl QueryId {
    pub(crate) fn new(queries_id: usize, id: usize) -> Self {
        Self { queries_id, id }
    }
}

/// This is a query that expects a sequence that contains exactly one single item.
///
/// Construct this using [`Queries::one`].
///
/// If it's empty or has more than one item, an error is returned.
///
/// The resulting item is converted into a Rust value using the `convert` function
/// when constructing this query.
///
/// This is useful if you expect a single item to be returned from an XPath query.
#[derive(Debug, Clone, Copy)]
pub struct OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    pub(crate) query_id: QueryId,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

fn execute_many(
    session: &mut Session,
    query_id: &QueryId,
    item: impl Itemable,
) -> Result<sequence::Sequence> {
    assert_eq!(query_id.queries_id, session.queries.id);
    let program = &session.queries.xpath_programs[query_id.id];
    let runnable = program.runnable(&session.dynamic_context);
    let item = item.to_item(session)?;
    runnable.many(Some(&item), &mut session.documents.xot)
}

impl<V, F> OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    /// Execute the query against an itemable.
    pub fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let mut items = sequence.items()?;
        let item = items.one()?;
        (self.convert)(session, &item)
    }
}

impl<V, F> Query<V, F> for OneQuery<V, F>
where
    F: Convert<V> + Copy,
{
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V> {
        OneQuery::execute(self, session, item)
    }
}

/// This is a query that expects an optional single item.
///
/// Construct this using ['Queries::option'].
///
/// If the sequence is empty, `None` is returned. If it contains more than one
/// item, an error is returned.
///
/// The result is converted into a Rust value using the `convert` function
/// when constructing this query.
///
/// This is useful if you expect an optional single item to be returned from an XPath query.
#[derive(Debug, Clone, Copy)]
pub struct OptionQuery<V, F>
where
    F: Convert<V> + Copy,
{
    pub(crate) query_id: QueryId,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F> OptionQuery<V, F>
where
    F: Convert<V> + Copy,
{
    /// Execute the query against an itemable.
    pub fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<Option<V>> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let mut items = sequence.items()?;
        let item = items.option()?;
        item.map(|item| (self.convert)(session, &item)).transpose()
    }
}

/// A query that expects many items as a result.
///
/// Construct this using [`Queries::many`].
///
/// The items are converted into Rust values using the supplied `convert` function.
///
/// This is useful if you expect many items to be returned from an XPath query.
///
/// The result is converted into a Rust value using the `convert` function
/// when constructing this query.
#[derive(Debug, Clone)]
pub struct ManyQuery<V, F>
where
    F: Convert<V> + Copy,
{
    pub(crate) query_id: QueryId,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F> ManyQuery<V, F>
where
    F: Convert<V> + Copy,
{
    /// Execute the query against an itemable.
    pub fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<Vec<V>> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let items = sequence
            .items()?
            .map(|item| (self.convert)(session, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}
