#![allow(dead_code)]

extern crate num;
#[macro_use]
extern crate num_derive;

mod atomic;
mod comparison;
mod context;
mod error;
mod func;
mod interpreter;
mod ir;
mod occurrence;
mod op;
mod output;
mod query;
mod run;
mod span;
mod stack;
mod types;
mod xml;
mod xpath;

pub use xee_xpath_ast::ast::Name;
pub use xee_xpath_ast::Namespaces;

pub use crate::atomic::Atomic;
pub use crate::context::{DynamicContext, StaticContext};
pub use crate::error::{Error, Result};
pub use crate::occurrence::Occurrence;
pub use crate::output::{Item, Sequence};
pub use crate::query::{
    Convert, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{evaluate, evaluate_root, evaluate_without_focus};
pub use crate::xml::Node;
pub use crate::xpath::XPath;
