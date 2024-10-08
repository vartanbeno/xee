//! Queries you can execute against a session.

use std::rc::Rc;

use xee_interpreter::context::{self, StaticContextRef};
use xee_interpreter::error::SpannedResult as Result;
use xee_interpreter::function;
use xee_interpreter::interpreter::Program;
use xee_interpreter::occurrence::Occurrence;
use xee_interpreter::sequence::{Item, Sequence};
use xee_xpath_compiler::Runnable;

use crate::{Itemable, Session};

// import only for documentation purposes
#[cfg(doc)]
use crate::context::DynamicContextBuilder;
#[cfg(doc)]
use crate::Queries;

/// A query that can be executed against an [`Itemable`]
///
/// It gives back a result of type `V`
pub trait Query<V>: GetProgram {
    /// Get the static context for the query.
    fn static_context(&self) -> StaticContextRef<'_>;

    // /// Get the signature for a given function.
    // fn signature(&self, session: &Session, function: &function::Function) -> &function::Signature {
    //     let context = self.dynamic_context_builder(session).build();
    //     let runnable = self.program().runnable(&context);
    //     runnable.function_info(function).signature()
    // }

    /// Execute the query against a dynamic context
    ///
    /// You can construct one using a [`DynamicContextBuilder`]
    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<V>;

    /// Get a dynamic context builder for the query, configured with the
    /// query's static context and the session's documents.
    ///
    /// You can use this if you want to construct your own dynamic context
    /// to use with `execute_with_context`.
    fn dynamic_context_builder(&self, session: &Session) -> context::DynamicContextBuilder {
        let mut context = context::DynamicContextBuilder::new(self.static_context().clone());
        context.documents(session.documents.clone());
        context
    }

    /// Map the the result of the query to a different type.
    ///
    /// You need to provide a function that takes the result of the query,
    /// the session, and the item, and returns a new result.
    fn map<T, F>(self, f: F) -> MapQuery<V, T, Self, F>
    where
        Self: Sized,
        F: Fn(V, &mut Session, &context::DynamicContext<'_>) -> Result<T> + Clone,
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
        let context_item = item.to_item(session)?;
        self.execute_build_context(session, move |builder| {
            builder.context_item(context_item);
        })
    }

    /// Execute a query with a specific dynamic context.
    ///
    /// This is useful if you want to build a dynamic context with specific
    /// settings (such as variables), and then execute a query against it.
    fn execute_build_context(
        &self,
        session: &mut Session,
        build: impl FnOnce(&mut context::DynamicContextBuilder),
    ) -> Result<V> {
        let mut dynamic_context_builder = self.dynamic_context_builder(session);
        build(&mut dynamic_context_builder);
        let context = dynamic_context_builder.build();
        self.execute_with_context(session, &context)
    }
}

trait GetProgram {
    fn program(&self) -> &Program;
}

/// A recursive query that can be executed against an [`Itemable`]
///
/// It gives back a result of type `V`
pub trait RecurseQuery<C, V> {
    /// Get the static context for the query.
    fn static_context(&self) -> StaticContextRef<'_>;

    /// Execute the query against an itemable, with context.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<C>;

    /// Get a dynamic context builder for the query, configured with the
    /// query's static context and the session's documents.
    ///
    /// You can use this if you want to construct your own dynamic context
    /// to use with `execute_with_context`.
    fn dynamic_context_builder(&self, session: &Session) -> context::DynamicContextBuilder {
        let mut context = context::DynamicContextBuilder::new(self.static_context().clone());
        context.documents(session.documents.clone());
        context
    }

    /// Execute the query against an itemable.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    fn execute(&self, session: &mut Session, item: &Item, recurse: &Recurse<V>) -> Result<C> {
        self.execute_build_context(session, recurse, |builder| {
            builder.context_item(item.clone());
        })
    }

    /// Execute a query against an itemable, building the context.
    ///
    /// This is useful if you want to build a dynamic context with specific
    /// settings (such as variables), and then execute a query against it.
    fn execute_build_context(
        &self,
        session: &mut Session,
        recurse: &Recurse<V>,
        build: impl FnOnce(&mut context::DynamicContextBuilder),
    ) -> Result<C> {
        let mut dynamic_context_builder = self.dynamic_context_builder(session);
        build(&mut dynamic_context_builder);
        let context = dynamic_context_builder.build();
        self.execute_with_context(session, &context, recurse)
    }
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
#[derive(Debug, Clone)]
pub struct OneQuery<'a, V, F>
where
    F: Convert<V>,
{
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<'a, V, F> GetProgram for OneQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a, V, F> OneQuery<'a, V, F>
where
    F: Convert<V>,
{
    /// Execute the query against a context
    pub fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<V> {
        let sequence = self.program.runnable(context).many(session.xot_mut())?;
        let mut items = sequence.items()?;
        let item = items.one()?;
        (self.convert)(session, &item)
    }
}

impl<'a, V, F> Query<V> for OneQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<V> {
        OneQuery::execute_with_context(self, session, context)
    }
}

/// A recursive query that expects a single item as a result.
#[derive(Debug, Clone)]
pub struct OneRecurseQuery<'a> {
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
}

impl<'a> GetProgram for OneRecurseQuery<'a> {
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a> OneRecurseQuery<'a> {
    /// Execute the query against an itemable, with context.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_context<V>(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<V> {
        let sequence = self.program.runnable(context).many(session.xot_mut())?;
        let mut items = sequence.items()?;
        let item = items.one()?;
        recurse.execute(session, &item)
    }
}

impl<'a, V> RecurseQuery<V, V> for OneRecurseQuery<'a> {
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<V> {
        OneRecurseQuery::execute_with_context(self, session, context, recurse)
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
#[derive(Debug, Clone)]
pub struct OptionQuery<'a, V, F>
where
    F: Convert<V>,
{
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<'a, V, F> GetProgram for OptionQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a, V, F> OptionQuery<'a, V, F>
where
    F: Convert<V>,
{
    /// Execute the query against an itemable, with explicit
    /// dynamic context.
    pub fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<Option<V>> {
        let sequence = self.program.runnable(context).many(session.xot_mut())?;
        let mut items = sequence.items()?;
        let item = items.option()?;
        item.map(|item| (self.convert)(session, &item)).transpose()
    }
}

impl<'a, V, F> Query<Option<V>> for OptionQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<Option<V>> {
        Self::execute_with_context(self, session, context)
    }
}

/// A recursive query that expects an optional single item.
#[derive(Debug, Clone)]
pub struct OptionRecurseQuery<'a> {
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
}

impl<'a> GetProgram for OptionRecurseQuery<'a> {
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a> OptionRecurseQuery<'a> {
    /// Execute the recursive query against an explicit dynamic context.
    pub fn execute_with_context<V>(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<Option<V>> {
        let sequence = self.program.runnable(context).many(session.xot_mut())?;
        let mut items = sequence.items()?;
        let item = items.option()?;
        item.map(|item| recurse.execute(session, &item)).transpose()
    }
}

impl<'a, V> RecurseQuery<Option<V>, V> for OptionRecurseQuery<'a> {
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<Option<V>> {
        OptionRecurseQuery::execute_with_context(self, session, context, recurse)
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
pub struct ManyQuery<'a, V, F>
where
    F: Convert<V>,
{
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
    pub(crate) convert: F,
    pub(crate) phantom: std::marker::PhantomData<V>,
}

impl<'a, V, F> GetProgram for ManyQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a, V, F> ManyQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<Vec<V>> {
        let sequence = self.program.runnable(context).many(session.xot_mut())?;
        let items = sequence
            .items()?
            .map(|item| (self.convert)(session, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}

impl<'a, V, F> Query<Vec<V>> for ManyQuery<'a, V, F>
where
    F: Convert<V>,
{
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<Vec<V>> {
        Self::execute_with_context(self, session, context)
    }
}

/// A recursive query that expects many items as a result.
#[derive(Debug, Clone)]
pub struct ManyRecurseQuery<'a> {
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
}

impl<'a> GetProgram for ManyRecurseQuery<'a> {
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a> ManyRecurseQuery<'a> {
    /// Execute the query against an itemable, with variables.
    ///
    /// To do the conversion pass in a [`Recurse`] object. This
    /// allows you to use a convert function recursively.
    pub fn execute_with_context<V>(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<Vec<V>> {
        let sequence = self.program.runnable(context).many(session.xot_mut())?;
        let items = sequence
            .items()?
            .map(|item| recurse.execute(session, &item))
            .collect::<Result<Vec<V>>>()?;
        Ok(items)
    }
}

impl<'a, V> RecurseQuery<Vec<V>, V> for ManyRecurseQuery<'a> {
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
        recurse: &Recurse<V>,
    ) -> Result<Vec<V>> {
        ManyRecurseQuery::execute_with_context(self, session, context, recurse)
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
#[derive(Debug, Clone)]
pub struct SequenceQuery<'a> {
    pub(crate) static_context: StaticContextRef<'a>,
    pub(crate) program: Rc<Program>,
}

impl<'a> GetProgram for SequenceQuery<'a> {
    fn program(&self) -> &Program {
        &self.program
    }
}

impl<'a> SequenceQuery<'a> {
    /// Execute the query against an itemable with an explict dynamic context.
    pub fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<Sequence> {
        self.program.runnable(context).many(session.xot_mut())
    }
}

impl<'a> Query<Sequence> for SequenceQuery<'a> {
    fn static_context(&self) -> StaticContextRef<'a> {
        self.static_context.clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<Sequence> {
        Self::execute_with_context(self, session, context)
    }
}

/// A query maps the result of another query to a different type.
#[derive(Debug, Clone)]
pub struct MapQuery<V, T, Q: Query<V> + Sized, F>
where
    F: Fn(V, &mut Session, &context::DynamicContext<'_>) -> Result<T> + Clone,
{
    query: Q,
    f: F,
    v: std::marker::PhantomData<V>,
    t: std::marker::PhantomData<T>,
}

impl<V, T, Q: Query<V> + Sized, F> GetProgram for MapQuery<V, T, Q, F>
where
    F: Fn(V, &mut Session, &context::DynamicContext<'_>) -> Result<T> + Clone,
{
    fn program(&self) -> &Program {
        &self.query.program()
    }
}

impl<V, T, Q, F> MapQuery<V, T, Q, F>
where
    Q: Query<V> + Sized,
    F: Fn(V, &mut Session, &context::DynamicContext<'_>) -> Result<T> + Clone,
{
    /// Execute the query against an item.
    pub fn execute(&self, session: &mut Session, item: &Item) -> Result<T> {
        let mut dynamic_context_builder =
            context::DynamicContextBuilder::new(self.query.static_context().clone());
        dynamic_context_builder.context_item(item.clone());
        let context = dynamic_context_builder.build();
        self.execute_with_context(session, &context)
    }

    /// Execute the query against a dynamic context.
    pub fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<T> {
        let v = self.query.execute_with_context(session, context)?;
        // TODO: this isn't right. need to rewrite in terms of dynamic context too?
        (self.f)(v, session, context)
    }
}

impl<V, T, Q: Query<V> + Sized, F> Query<T> for MapQuery<V, T, Q, F>
where
    F: Fn(V, &mut Session, &context::DynamicContext<'_>) -> Result<T> + Clone,
{
    fn static_context(&self) -> StaticContextRef<'_> {
        self.query.static_context().clone()
    }

    fn execute_with_context(
        &self,
        session: &mut Session,
        context: &context::DynamicContext,
    ) -> Result<T> {
        let v = self.query.execute_with_context(session, context)?;
        (self.f)(v, session, context)
    }
}
