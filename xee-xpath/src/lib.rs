#![warn(missing_docs)]

//! This module provides a high level API to use XPath from Rust.
//!
//! You can create a [`Documents`] store and load documents into it. You can
//! also compile XPath expressions using the [`XPaths`] store.
//!
//! You can then construct an [`Engine`] to execute the compiled XPath
//! expressions against documents.
pub mod atomic;
mod documents;
mod query;
mod sequence;

pub use documents::{DocumentHandle, Documents};
pub use query::Queries;
pub use sequence::Sequence;
pub use xee_interpreter::atomic::Atomic;
pub use xee_interpreter::error::{Result, SpannedResult};
pub use xee_interpreter::sequence::Item;
