#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate num;
#[macro_use]
extern crate num_derive;

mod ast;
mod ast_interpret;
mod builder;
mod error;
mod instruction;
mod interpret;
mod name;
mod parse;
mod parse_ast;
mod scope;
mod static_context;
mod step;
mod value;
