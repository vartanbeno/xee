mod builder;
mod instruction;
mod interpret;
mod ir_interpret;
mod scope;

pub(crate) use builder::FunctionBuilder;
pub(crate) use interpret::Interpreter;
pub(crate) use ir_interpret::{InterpreterCompiler, Scopes};
