mod builder;
mod interpret;
mod ir_interpret;

pub(crate) use builder::{FunctionBuilder, Program};
pub(crate) use interpret::Interpreter;
pub(crate) use ir_interpret::{InterpreterCompiler, Scopes};
