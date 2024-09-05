#![warn(missing_docs)]

//! This module provides a high level API to use XPath from Rust.
//!
//! You can create a [`Documents`] store and load documents into it. You can
//! also compile XPath expressions using the [`XPaths`] store.
//!
//! You can then construct an [`Engine`] to execute the compiled XPath
//! expressions against documents.
mod high_level;
mod sequence;

pub mod atomic;

pub use high_level::{DocumentHandle, Documents, Engine, XPathHandle, XPaths};
pub use sequence::Sequence;
pub use xee_interpreter::atomic::Atomic;
pub use xee_interpreter::sequence::Item;
