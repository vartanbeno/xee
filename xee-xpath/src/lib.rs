#![allow(dead_code)]

#[macro_use]
extern crate num_derive;

mod atomic;
pub mod context;
pub mod error;
pub mod function;
pub mod interpreter;
pub mod ir;
mod library;
mod occurrence;
mod query;
mod run;
mod sequence;
pub mod span;
pub mod stack;
mod string;
pub mod xml;

pub use xee_xpath_ast::ast::Name;
pub use xee_xpath_ast::{Namespaces, VariableNames};

pub use crate::atomic::Atomic;
pub use crate::atomic::{
    Duration, GDay, GMonth, GMonthDay, GYear, GYearMonth, NaiveDateTimeWithOffset,
    NaiveDateWithOffset, NaiveTimeWithOffset, YearMonthDuration,
};
pub use crate::context::{DynamicContext, StaticContext, Variables};
pub use crate::error::{Error, Result, SpannedError, SpannedResult};
pub use crate::interpreter::{Program, Runnable};
pub use crate::occurrence::Occurrence;
pub use crate::query::{
    Convert, ManyQuery, OneQuery, OptionQuery, Queries, Query, Recurse, Session,
};
pub use crate::run::{
    evaluate, evaluate_root, evaluate_without_focus, evaluate_without_focus_with_variables,
};
pub use crate::sequence::{Item, Sequence};
pub use crate::string::Collation;
pub use crate::xml::{Document, Documents, Node, Uri};
