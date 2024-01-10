mod builder;
pub mod instruction;
mod interpret;
mod ir_interpret;
mod program;
mod runnable;
mod scope;
mod state;

pub use interpret::Interpreter;
pub use program::Program;
pub use runnable::Runnable;
