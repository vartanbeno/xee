#![warn(missing_docs)]

//! This module provides a high level API to use XPath from Rust.
//!
//! You can compile XPath expressions into queries using the [`Queries`] store.
//! For each query you supply a conversion function that turns the result XPath
//! sequence into a Rust value.
//!
//! You can create a [`Documents`] store and load XML documents into it.
//!
//! You can then combine the queries and documents into a [`Session`]. You use
//! this to execute queries.
//!
//! ```rust
//! use xee_xpath::{Documents, Queries, Query, Uri};
//!
//! // create a new documents object
//! let mut documents = Documents::new();
//! // load a document from a string
//! let doc = documents.add_string(&Uri::new("http://example.com"), "<root>foo</root>").unwrap();
//!
//! // create a new queries object
//! let mut queries = Queries::default();
//!
//! // create a query expecting a single value in the result sequence
//! // try to convert this value into a Rust `String`
//! let q = queries.one("/root/string()", |_, item| {
//!   Ok(item.try_into_value::<String>()?)
//! })?;
//!
//! // now create a session for the documents and execute the query
//! let mut session = queries.session(documents);
//!
//! // when we execute the query, we need to pass the session, and the item
//! // against which we want to query. We can also pass in a document handle,
//! // as we do here
//! let r = q.execute(&mut session, doc)?;
//! assert_eq!(r, "foo");
//!
//! # Ok::<(), xee_xpath::error::Error>(())
//! ```
pub mod atomic;
mod documents;
pub mod error;
mod itemable;
pub mod iter;
mod queries;
pub mod query;
mod session;

pub use documents::Documents;
pub use itemable::Itemable;
pub use queries::Queries;
pub use query::{Query, Recurse};
pub use session::Session;
pub use xee_interpreter::atomic::Atomic;
pub use xee_interpreter::context::{DynamicContextBuilder, StaticContextBuilder, Variables};
pub use xee_interpreter::function::{Array, Map};
pub use xee_interpreter::sequence::{Item, Sequence};
pub use xee_interpreter::xml::{DocumentHandle, Uri};
