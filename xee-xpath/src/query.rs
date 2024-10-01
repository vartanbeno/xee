//! Queries you can execute against a session.

use xee_interpreter::context::Variables;
use xee_interpreter::error::SpannedResult as Result;
use xee_interpreter::occurrence::Occurrence;
use xee_interpreter::sequence::{self, Item, Sequence};

use crate::session::Session;
use crate::{error, Itemable};

// import only for documentation purposes
#[cfg(doc)]
use crate::Queries;

/// A query that can be executed against an [`Itemable`]
///
/// It gives back a result of type `V`
pub trait Query<V> {
    /// Map the the result of the query to a different type.
    ///
    /// You need to provide a function that takes the result of the query,
    /// the session, and the item, and returns a new result.
    fn map<T, F>(self, f: F) -> MapQuery<V, T, Self, F>
    where
        Self: Sized,
        F: Fn(V, &mut Session, Option<&Item>) -> Result<T> + Clone,
    {
        MapQuery {
            query: self,
            f,
            v: std::marker::PhantomData,
            t: std::marker::PhantomData,
        }
    }

    /// Excute the query against an itemable
    fn execute(&self, session: &mut Session, item: impl Itemable) -> Result<V> {
        self.execute_with_variables(session, Some(item))
    }

    /// Execute the query without a context item
    fn execute_without_context<T: Itemable>(&self, session: &mut Session) -> Result<V> {
        self.execute_with_variables(session, None::<T>)
    }

    /// Execute the query against an optional itemable, with variables
    ///
    /// This is also useful in a [`MapQuery`] invocation where you get an
    /// Option<Item> in your closure.
    fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<V>;
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

// The dyn and reference are unavoidable, as closures are not allowed
// to refer to themselves:
// https://github.com/rust-lang/rust/issues/46062
type RecurseFn<'s, V> = &'s dyn Fn(&mut Session, &Item, &Recurse<'s, V>) -> Result<V>;

/// An object that can be used to use a conversion function recursively.
pub struct Recurse<'s, V> {
    f: RecurseFn<'s, V>,
}

impl<'s, V> Recurse<'s, V> {
    /// Create a new recurse object given a conversion function.
    pub fn new(f: RecurseFn<'s, V>) -> Self {
        Self { f }
    }

    /// Execute the conversion function against an item.
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
    F: Convert<V>,
{
    pub(crate) query_id: QueryId,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

fn execute_many(
    session: &mut Session,
    query_id: &QueryId,
    item: Option<impl Itemable>,
) -> Result<sequence::Sequence> {
    if query_id.queries_id != session.queries.id {
        return Err(error::ErrorValue::UsedQueryWithWrongQueries.into());
    }
    let program = &session.queries.xpath_programs[query_id.id];
    let mut dynamic_context_builder = session.dynamic_context_builder.clone();
    if let Some(item) = item {
        dynamic_context_builder.context_item(item.to_item(session)?);
    }

    let dynamic_context = dynamic_context_builder.build();

    let runnable = program.runnable(&dynamic_context);

    runnable.many(&mut session.xot)
}

impl<V, F> OneQuery<V, F>
where
    F: Convert<V>,
{
    /// Execute the query against an itemable.
    pub fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<V> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let mut items = sequence.items()?;
        let item = items.one()?;
        (self.convert)(session, &item)
    }
}

impl<V, F> Query<V> for OneQuery<V, F>
where
    F: Convert<V>,
{
    fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<V> {
        OneQuery::execute_with_variables(self, session, item)
    }
}

/// A recursive query that expects a single item as a result.
#[derive(Debug, Clone)]
pub struct OneRecurseQuery {
    pub(crate) query_id: QueryId,
}

impl OneRecurseQuery {
    /// Execute the query against an itemable.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute<V>(
        &self,
        session: &mut Session,
        item: &Item,
        recurse: &Recurse<V>,
    ) -> Result<V> {
        self.execute_with_variables(session, Some(item), recurse)
    }

    /// Execute the query against an itemable, with variables.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_variables<V>(
        &self,
        session: &mut Session,
        item: Option<&Item>,
        recurse: &Recurse<V>,
    ) -> Result<V> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let mut items = sequence.items()?;
        let item = items.one()?;
        recurse.execute(session, &item)
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
    F: Convert<V>,
{
    pub(crate) query_id: QueryId,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F> OptionQuery<V, F>
where
    F: Convert<V>,
{
    /// Execute the query against an itemable, with variables.
    pub fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<Option<V>> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let mut items = sequence.items()?;
        let item = items.option()?;
        item.map(|item| (self.convert)(session, &item)).transpose()
    }
}

impl<V, F> Query<Option<V>> for OptionQuery<V, F>
where
    F: Convert<V>,
{
    fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<Option<V>> {
        Self::execute_with_variables(self, session, item)
    }
}

/// A recursive query that expects an optional single item.
#[derive(Debug, Clone)]
pub struct OptionRecurseQuery {
    pub(crate) query_id: QueryId,
}

impl OptionRecurseQuery {
    /// Execute the query against an itemable.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute<V>(
        &self,
        session: &mut Session,
        item: &Item,
        recurse: &Recurse<V>,
    ) -> Result<Option<V>> {
        self.execute_with_variables(session, Some(item), recurse)
    }

    /// Execute the query against an itemable, with variables
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_variables<V>(
        &self,
        session: &mut Session,
        item: Option<&Item>,
        recurse: &Recurse<V>,
    ) -> Result<Option<V>> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let mut items = sequence.items()?;
        let item = items.option()?;
        item.map(|item| recurse.execute(session, &item)).transpose()
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
#[derive(Debug, Clone, Copy)]
pub struct ManyQuery<V, F>
where
    F: Convert<V>,
{
    pub(crate) query_id: QueryId,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<V, F> ManyQuery<V, F>
where
    F: Convert<V>,
{
    /// Execute the query against an itemable.
    pub fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<Vec<V>> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let items = sequence
            .items()?
            .map(|item| (self.convert)(session, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}

impl<V, F> Query<Vec<V>> for ManyQuery<V, F>
where
    F: Convert<V>,
{
    fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<Vec<V>> {
        Self::execute_with_variables(self, session, item)
    }
}

/// A recursive query that expects many items as a result.
#[derive(Debug, Clone)]
pub struct ManyRecurseQuery {
    pub(crate) query_id: QueryId,
}

impl ManyRecurseQuery {
    /// Execute the query against an itemable.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute<V>(
        &self,
        session: &mut Session,
        item: &Item,
        recurse: &Recurse<V>,
    ) -> Result<Vec<V>> {
        self.execute_with_variables(session, Some(item), recurse)
    }

    /// Execute the query against an itemable, with variables.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_variables<V>(
        &self,
        session: &mut Session,
        item: Option<&Item>,
        recurse: &Recurse<V>,
    ) -> Result<Vec<V>> {
        let sequence = execute_many(session, &self.query_id, item)?;
        let items = sequence
            .items()?
            .map(|item| recurse.execute(session, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}

/// A query that returns a sequence.
///
/// This query returns a [`Sequence`] object that can be used to access
/// the items in the sequence. It represents an XPath sequence. The items
/// in the sequence are not converted.
///
/// Construct this using [`Queries::sequence`].
///
/// This is useful if you want to work with the sequence directly.
#[derive(Debug, Clone, Copy)]
pub struct SequenceQuery {
    pub(crate) query_id: QueryId,
}

impl SequenceQuery {
    /// Execute the query against an itemable.
    pub fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<Sequence> {
        execute_many(session, &self.query_id, item)
    }
}

impl Query<Sequence> for SequenceQuery {
    fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<Sequence> {
        Self::execute_with_variables(self, session, item)
    }
}

/// A query maps the result of another query to a different type.
#[derive(Debug, Copy, Clone)]
pub struct MapQuery<V, T, Q: Query<V> + Sized, F>
where
    F: Fn(V, &mut Session, Option<&Item>) -> Result<T> + Clone,
{
    query: Q,
    f: F,
    v: std::marker::PhantomData<V>,
    t: std::marker::PhantomData<T>,
}

impl<V, T, Q, F> MapQuery<V, T, Q, F>
where
    Q: Query<V> + Sized,
    F: Fn(V, &mut Session, Option<&Item>) -> Result<T> + Clone,
{
    /// Execute the query against an item.
    pub fn execute(&self, session: &mut Session, item: &Item) -> Result<T> {
        let v = self.query.execute(session, item)?;
        (self.f)(v, session, Some(item))
    }
}

impl<V, T, Q: Query<V> + Sized, F> Query<T> for MapQuery<V, T, Q, F>
where
    F: Fn(V, &mut Session, Option<&Item>) -> Result<T> + Clone,
{
    fn execute_with_variables(
        &self,
        session: &mut Session,
        item: Option<impl Itemable>,
    ) -> Result<T> {
        let item = if let Some(item) = item {
            Some(item.to_item(session)?)
        } else {
            None
        };
        let v = self.query.execute_with_variables(session, item.as_ref())?;
        (self.f)(v, session, item.as_ref())
    }
}
