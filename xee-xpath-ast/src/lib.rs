#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;
mod error;
mod lexer;
mod namespaces;
mod operator;
mod parser;
pub mod span;

pub use error::Error;
pub use namespaces::{Namespaces, FN_NAMESPACE, XS_NAMESPACE};
