use xee_interpreter::error::SpannedResult as Result;
use xee_xpath_compiler::parse;

use std::sync::atomic;

use crate::{
    documents::Documents,
    query::{Convert, ManyQuery, OneQuery, OptionQuery, OptionRecurseQuery, QueryId},
    session::Session,
};

static QUERIES_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

fn get_queries_id() -> usize {
    QUERIES_COUNTER.fetch_add(1, atomic::Ordering::SeqCst)
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

    /// Construct a [`Session`] with a collection of documents
    ///
    /// You need a session to be able to execute queries against documents.
    pub fn session(&self, documents: Documents) -> Session {
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
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(OneQuery {
            query_id: QueryId::new(self.id, id),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    // pub fn one_recurse(&mut self, s: &str) -> Result<OneRecurseQuery> {
    //     let id = self.register(s)?;
    //     Ok(OneRecurseQuery {
    //         queries_id: self.id,
    //         id,
    //     })
    // }

    // pub fn one_recurse(&mut self, s: &str) -> Result<OneRecurseQuery> {
    //     let id = self.register(s)?;
    //     Ok(OneRecurseQuery { id })
    // }

    /// Connstruct a query that expects an optional single item result.
    ///
    /// This item is converted into a Rust value using supplied `convert` function.
    pub fn option<V, F>(&mut self, s: &str, convert: F) -> Result<OptionQuery<V, F>>
    where
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(OptionQuery {
            query_id: QueryId::new(self.id, id),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

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
        F: Convert<V> + Copy,
    {
        let id = self.register(s)?;
        Ok(ManyQuery {
            query_id: QueryId::new(self.id, id),
            convert,
            phantom: std::marker::PhantomData,
        })
    }

    // pub fn many_recurse(&mut self, s: &str) -> Result<ManyRecurseQuery> {
    //     let id = self.register(s)?;
    //     Ok(ManyRecurseQuery { id })queries
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_one_query() -> Result<()> {
        let mut documents = Documents::new();
        let doc = documents
            .load_string("http://example.com", "<root>foo</root>")
            .unwrap();

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
