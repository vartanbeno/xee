mod compile;

#[cfg(test)]
pub(crate) use compile::convert_ir;
pub use compile::{compile, parse};
