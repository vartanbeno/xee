#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate num;
#[macro_use]
extern crate num_derive;

mod annotation;
mod ast;
mod ast_ir;
mod builder;
mod document;
mod dynamic_context;
mod error;
mod instruction;
mod interpret;
mod ir;
mod ir_interpret;
mod name;
mod op;
mod operator;
mod parse;
mod parse_ast;
mod query;
mod run;
mod scope;
mod span;
mod static_context;
mod step;
mod value;
mod xpath;

pub use crate::dynamic_context::DynamicContext;
pub use crate::error::Error;
pub use crate::name::Namespaces;
pub use crate::query::{
    Convert, ConvertError, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{evaluate, evaluate_root, evaluate_without_focus};
pub use crate::static_context::StaticContext;
pub use crate::value::{Atomic, Item, Node, Sequence, StackValue};
pub use crate::xpath::XPath;
