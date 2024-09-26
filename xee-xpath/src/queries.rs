use xee_interpreter::error::SpannedResult as Result;
use xee_xpath_compiler::parse;

use std::sync::atomic;

use crate::{
    documents::OwnedDocuments,
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
pub struct Queries<'namespaces> {
    pub(crate) id: usize,
    pub(crate) xpath_programs: Vec<xee_interpreter::interpreter::Program>,
    pub(crate) static_context: xee_interpreter::context::StaticContext<'namespaces>,
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

    /// Construct a new collection of queries with a default namespace for XPath
    pub fn with_default_namespace(default_ns: &'namespaces str) -> Self {
        let namespaces = xee_xpath_ast::Namespaces::new(
            xee_xpath_ast::Namespaces::default_namespaces(),
            default_ns,
            xee_xpath_ast::FN_NAMESPACE,
        );
        let static_context = xee_interpreter::context::StaticContext::from_namespaces(namespaces);
        Self::new(static_context)
    }

    /// Construct a [`Session`] with a collection of documents
    ///
    /// You need a session to be able to execute queries against documents.
    pub fn session(&self, documents: OwnedDocuments) -> Session {
        Session::new(self, documents)
    }

    fn register(&mut self, s: &str) -> Result<usize> {
        let program = parse(&self.static_context, s)?;
        let id = self.xpath_programs.len();
        self.xpath_programs.push(program);
        Ok(id)
    }

    /// Construct a query that expects a single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    pub fn one<V, F>(&mut self, s: &str, convert: F) -> Result<OneQuery<V, F>>
    where
        F: Convert<V>,
    {
        let id = self.register(s)?;
        Ok(OneQuery {
            query_id: QueryId::new(self.id, id),
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
    pub fn one_recurse(&mut self, s: &str) -> Result<OneRecurseQuery> {
        let id = self.register(s)?;
        Ok(OneRecurseQuery {
            query_id: QueryId::new(self.id, id),
        })
    }

    /// Connstruct a query that expects an optional single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    pub fn option<V, F>(&mut self, s: &str, convert: F) -> Result<OptionQuery<V, F>>
    where
        F: Convert<V>,
    {
        let id = self.register(s)?;
        Ok(OptionQuery {
            query_id: QueryId::new(self.id, id),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    /// Construct a query that expects an optional single item result.
    ///
    /// This item is converted into a Rust value not using a convert
    /// function but through a recursive call that's passed in during
    /// execution.
    pub fn option_recurse(&mut self, s: &str) -> Result<OptionRecurseQuery> {
        let id = self.register(s)?;
        Ok(OptionRecurseQuery {
            query_id: QueryId::new(self.id, id),
        })
    }

    /// Construct a query that expects many items as a result.
    ///
    /// These items are converted into Rust values using supplied `convert` function.
    pub fn many<V, F>(&mut self, s: &str, convert: F) -> Result<ManyQuery<V, F>>
    where
        F: Convert<V>,
    {
        let id = self.register(s)?;
        Ok(ManyQuery {
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
    pub fn many_recurse(&mut self, s: &str) -> Result<ManyRecurseQuery> {
        let id = self.register(s)?;
        Ok(ManyRecurseQuery {
            query_id: QueryId::new(self.id, id),
        })
    }

    /// Construct a query that gets a [`Sequence`] as a result.
    ///
    /// This is a low-level API that allows you to get the raw sequence
    /// without converting it into Rust values.
    pub fn sequence(&mut self, s: &str) -> Result<SequenceQuery> {
        let id = self.register(s)?;
        Ok(SequenceQuery {
            query_id: QueryId::new(self.id, id),
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
        let mut documents = OwnedDocuments::new();
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
