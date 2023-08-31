mod dynamic_context;
mod static_context;
mod static_function;

pub use dynamic_context::DynamicContext;
pub use static_context::StaticContext;

// we allow StaticFunctionType as it's used in the xpath_fn macro
#[allow(unused_imports)]
pub(crate) use static_function::StaticFunctionType;
pub(crate) use static_function::{FunctionKind, FunctionRule, StaticFunctionDescription};
