#![allow(dead_code)]

#[macro_use]
extern crate num_derive;

pub mod atomic;
pub mod context;
pub mod error;
pub mod function;
pub mod interpreter;
mod library;
pub mod occurrence;
pub mod sequence;
pub mod span;
pub mod stack;
pub mod string;
pub mod xml;

pub use xee_xpath_ast::ast::Name;
pub use xee_xpath_ast::{Namespaces, VariableNames};
