mod dynamic_context;
mod static_context;
mod static_function;

pub use dynamic_context::DynamicContext;
pub use static_context::StaticContext;
pub(crate) use static_function::{ContextRule, FunctionType, StaticFunctionDescription};
