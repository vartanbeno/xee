#![warn(missing_docs)]

//! This module provides a high level API to use XPath from Rust.
//!
//! You can compile XPath expressions using the [`Queries`] store. You
//! can use this to configure how to turn the result sequence into
//! a Rust value.
//!
//! You can create a [`Documents`] store and load documents into it,
//! and use this to create a [`Session`] to execute queries.
//!
//! ```rust
//! use xee_xpath::{Documents, Queries};
//!
//! // create a new documents object
//! let mut documents = Documents::new();
//! // load a document from a string
//! let doc = documents.load_string("http://example.com", "<root>foo</root>").unwrap();
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
mod queries;
mod query;
mod sequence;
mod session;

pub use documents::{DocumentHandle, Documents};
pub use itemable::Itemable;
pub use queries::Queries;
pub use query::{ManyQuery, OneQuery, OptionQuery, Query};
pub use sequence::Sequence;
pub use session::Session;
pub use xee_interpreter::atomic::Atomic;

pub use xee_interpreter::sequence::Item;
