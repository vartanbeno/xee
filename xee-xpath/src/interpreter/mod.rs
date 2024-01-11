mod builder;
mod compile;
mod ir_interpret;
mod scope;

#[cfg(test)]
pub(crate) use compile::convert_ir;
pub use compile::{compile, parse};
