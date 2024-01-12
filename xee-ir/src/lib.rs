mod builder;
pub mod ir;
mod ir_interpret;
mod scope;

pub use builder::FunctionBuilder;
pub use ir_interpret::InterpreterCompiler;
pub use scope::Scopes;
