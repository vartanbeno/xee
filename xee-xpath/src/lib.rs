#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate num;
#[macro_use]
extern crate num_derive;

mod ast;
mod ast_interpret;
mod interpret;
mod interpret2;
mod parse;
mod parse_ast;
