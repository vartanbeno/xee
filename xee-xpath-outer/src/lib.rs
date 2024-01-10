#![allow(dead_code)]

mod interpreter;
mod ir;
mod occurrence;
mod query;
mod run;

pub use xee_xpath_ast::ast::Name;
pub use xee_xpath_ast::{Namespaces, VariableNames};

pub use crate::occurrence::Occurrence;
pub use crate::query::{
    Convert, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{
    evaluate, evaluate_root, evaluate_without_focus, evaluate_without_focus_with_variables,
};
