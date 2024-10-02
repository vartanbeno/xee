use xee_interpreter::{context, error::SpannedResult as Result};
use xee_xpath_ast::VariableNames;
use xee_xpath_compiler::parse;

use std::{rc::Rc, sync::atomic};

use crate::{
    documents::Documents,
    query::{
        Convert, ManyQuery, ManyRecurseQuery, OneQuery, OneRecurseQuery, OptionQuery,
        OptionRecurseQuery, QueryId, SequenceQuery,
    },
    session::Session,
};

static QUERIES_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

fn get_queries_id() -> usize {
    QUERIES_COUNTER.fetch_add(1, atomic::Ordering::Relaxed)
}

/// A collection of XPath queries
///
/// You can register xpath expressions with conversion functions
/// to turn the results into Rust values.
#[derive(Debug)]
pub struct Queries<'a> {
    pub(crate) id: usize,
    pub(crate) default_static_context_builder: context::StaticContextBuilder<'a>,
    pub(crate) xpath_programs: Vec<xee_interpreter::interpreter::Program>,
}

impl Default for Queries<'_> {
    fn default() -> Self {
        Self {
            id: get_queries_id(),
            default_static_context_builder: context::StaticContextBuilder::default(),
            xpath_programs: Vec::new(),
        }
    }
}

impl<'a> Queries<'a> {
    /// Construct a new collection of queries
    ///
    /// Supply a default static context builder, which will be used
    /// by default to construct a static context if none is supplied
    /// explicitly.
    pub fn new(default_static_context_builder: context::StaticContextBuilder<'a>) -> Self {
        Self {
            id: get_queries_id(),
            default_static_context_builder,
            xpath_programs: Vec::new(),
        }
    }

    /// Construct a [`Session`] with a collection of documents
    ///
    /// You need a session to be able to execute queries against documents.
    pub fn session(&self, documents: Documents) -> Session {
        Session::new(self, documents)
    }

    fn register(&mut self, s: &str, static_context: &context::StaticContext<'a>) -> Result<usize> {
        let program = parse(&static_context, s)?;
        let id = self.xpath_programs.len();
        self.xpath_programs.push(program);
        Ok(id)
    }

    /// Construct a query that expects a single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    ///
    /// This uses a default static context.
    pub fn one<V, F>(&mut self, s: &str, convert: F) -> Result<OneQuery<'a, V, F>>
    where
        F: Convert<V>,
    {
        self.one_with_context(s, convert, self.default_static_context_builder.build())
    }

    /// Construct a query that expects a single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    ///
    /// You can supply a static context explicitly.
    pub fn one_with_context<V, F>(
        &mut self,
        s: &str,
        convert: F,
        static_context: context::StaticContext<'a>,
    ) -> Result<OneQuery<'a, V, F>>
    where
        F: Convert<V>,
    {
        let id = self.register(s, &static_context)?;
        Ok(OneQuery {
            query_id: QueryId::new(self.id, id),
            static_context: Rc::new(static_context),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    /// Construct a query that expects a single item result.
    ///
    /// This item is converted into a Rust value not using a convert function
    /// but through a recursive call that's passed in during execution.
    ///
    /// NOTE: recursion generally needs a stopping condition, but `one_recurse`
    /// expects one value always - unlike `option_recurse` and `many_recurse`
    /// which have the None or empty value. I think this means that
    /// `one_recurse` is not in fact useful.
    pub fn one_recurse(&mut self, s: &str) -> Result<OneRecurseQuery<'a>> {
        self.one_recurse_with_context(s, self.default_static_context_builder.build())
    }

    pub fn one_recurse_with_context(
        &mut self,
        s: &str,
        static_context: context::StaticContext<'a>,
    ) -> Result<OneRecurseQuery<'a>> {
        let id = self.register(s, &static_context)?;
        Ok(OneRecurseQuery {
            query_id: QueryId::new(self.id, id),
            static_context: Rc::new(static_context),
        })
    }

    /// Connstruct a query that expects an optional single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    pub fn option<V, F>(&mut self, s: &str, convert: F) -> Result<OptionQuery<'a, V, F>>
    where
        F: Convert<V>,
    {
        self.option_with_context(s, convert, self.default_static_context_builder.build())
    }

    pub fn option_with_context<V, F>(
        &mut self,
        s: &str,
        convert: F,
        static_context: context::StaticContext<'a>,
    ) -> Result<OptionQuery<'a, V, F>>
    where
        F: Convert<V>,
    {
        let id = self.register(s, &static_context)?;
        Ok(OptionQuery {
            query_id: QueryId::new(self.id, id),
            static_context: Rc::new(static_context),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    /// Construct a query that expects an optional single item result.
    ///
    /// This item is converted into a Rust value not using a convert
    /// function but through a recursive call that's passed in during
    /// execution.
    pub fn option_recurse(&mut self, s: &str) -> Result<OptionRecurseQuery<'a>> {
        self.option_recurse_with_context(s, self.default_static_context_builder.build())
    }

    pub fn option_recurse_with_context(
        &mut self,
        s: &str,
        static_context: context::StaticContext<'a>,
    ) -> Result<OptionRecurseQuery<'a>> {
        let id = self.register(s, &static_context)?;
        Ok(OptionRecurseQuery {
            query_id: QueryId::new(self.id, id),
            static_context: Rc::new(static_context),
        })
    }

    /// Construct a query that expects many items as a result.
    ///
    /// These items are converted into Rust values using supplied `convert` function.
    pub fn many<V, F>(&mut self, s: &str, convert: F) -> Result<ManyQuery<'a, V, F>>
    where
        F: Convert<V>,
    {
        self.many_with_builder(s, convert, self.default_static_context_builder.build())
    }

    pub fn many_with_builder<V, F>(
        &mut self,
        s: &str,
        convert: F,
        static_context: context::StaticContext<'a>,
    ) -> Result<ManyQuery<'a, V, F>>
    where
        F: Convert<V>,
    {
        let id = self.register(s, &static_context)?;
        Ok(ManyQuery {
            static_context: Rc::new(static_context),
            query_id: QueryId::new(self.id, id),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    /// Construct a query that expects many items as a result.
    ///
    /// These items are converted into Rust values not using a convert
    /// function but through a recursive call that's passed in during
    /// execution.
    pub fn many_recurse(&mut self, s: &str) -> Result<ManyRecurseQuery<'a>> {
        self.many_recurse_with_context(s, self.default_static_context_builder.build())
    }

    pub fn many_recurse_with_context(
        &mut self,
        s: &str,
        static_context: context::StaticContext<'a>,
    ) -> Result<ManyRecurseQuery<'a>> {
        let id = self.register(s, &static_context)?;
        Ok(ManyRecurseQuery {
            query_id: QueryId::new(self.id, id),
            static_context: Rc::new(static_context),
        })
    }

    /// Construct a query that gets a [`Sequence`] as a result.
    ///
    /// This is a low-level API that allows you to get the raw sequence
    /// without converting it into Rust values.
    pub fn sequence(&mut self, s: &str) -> Result<SequenceQuery<'a>> {
        self.sequence_with_context(s, self.default_static_context_builder.build())
    }

    pub fn sequence_with_context(
        &mut self,
        s: &str,
        static_context: context::StaticContext<'a>,
    ) -> Result<SequenceQuery<'a>> {
        let id = self.register(s, &static_context)?;
        Ok(SequenceQuery {
            query_id: QueryId::new(self.id, id),
            static_context: Rc::new(static_context),
        })
    }
}

#[cfg(test)]
mod tests {

    use xee_interpreter::xml::Uri;

    use crate::query::Query;

    use super::*;

    #[test]
    fn test_one_query() -> Result<()> {
        let mut documents = Documents::new();
        let uri = Uri::new("http://example.com");
        let doc = documents.add_string(&uri, "<root>foo</root>").unwrap();

        let mut queries = Queries::default();
        let q = queries.one("/root/string()", |_, item| {
            Ok(item.try_into_value::<String>()?)
        })?;

        let mut session = queries.session(documents);
        let r = q.execute(&mut session, doc)?;
        assert_eq!(r, "foo");
        Ok(())
    }
}
