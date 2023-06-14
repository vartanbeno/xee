#![allow(dead_code)]

extern crate num;
#[macro_use]
extern crate num_derive;

mod annotation;
mod comparison;
mod context;
mod data;
mod document;
mod error;
mod func;
mod interpreter;
mod ir;
mod op;
mod query;
mod run;
mod span;
mod stack;
mod step;
mod types;
mod xpath;

pub use xee_xpath_ast::ast::Name;
pub use xee_xpath_ast::Namespaces;

pub use crate::context::{DynamicContext, StaticContext};
pub use crate::data::{Node, OutputAtomic, OutputItem, OutputSequence, ValueError, ValueResult};
pub use crate::error::Error;
pub use crate::query::{
    Convert, ConvertError, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{evaluate, evaluate_root, evaluate_without_focus};
pub use crate::xpath::XPath;
