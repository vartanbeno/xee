#![allow(dead_code)]

mod interpreter;
mod ir;
mod query;
mod run;

pub use xee_xpath_ast::ast::Name;
pub use xee_xpath_ast::{Namespaces, VariableNames};

pub use xee_interpreter::interpreter::Runnable;
pub use xee_interpreter::{atomic, context, error, occurrence, sequence, string, xml};

pub use crate::interpreter::{compile, parse};
pub use crate::query::{
    Convert, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{
    evaluate, evaluate_root, evaluate_without_focus, evaluate_without_focus_with_variables,
};
