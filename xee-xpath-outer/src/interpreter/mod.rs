mod builder;
mod compile;
mod ir_interpret;
mod scope;

pub(crate) use compile::convert_ir;
pub use compile::{compile, parse};
