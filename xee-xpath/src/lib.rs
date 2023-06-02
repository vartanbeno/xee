#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate num;
#[macro_use]
extern crate num_derive;

mod annotation;
mod ast;
mod comparison;
mod context;
mod document;
mod error;
mod interpreter;
mod ir;
mod op;
mod operator;
mod parser;
mod query;
mod run;
mod span;
mod step;
mod types;
mod value;
mod xpath;

pub use crate::ast::Name;
pub use crate::context::{DynamicContext, Namespaces, StaticContext};
pub use crate::error::Error;
pub use crate::query::{
    Convert, ConvertError, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{evaluate, evaluate_root, evaluate_without_focus};
pub use crate::value::{Atomic, Item, Node, Sequence, StackValue};
pub use crate::xpath::XPath;
