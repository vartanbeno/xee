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
mod context;
mod document;
mod error;
mod instruction;
mod interpret;
mod ir;
mod ir_interpret;
mod name;
mod parse;
mod parse_ast;
mod run;
mod scope;
mod static_context;
mod step;
mod value;
mod xpath;

pub use crate::run::evaluate;
