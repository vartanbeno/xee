mod dynamic_context;
mod namespaces;
mod static_context;

pub use dynamic_context::DynamicContext;
pub use namespaces::Namespaces;
pub(crate) use namespaces::{FN_NAMESPACE, XS_NAMESPACE};
pub(crate) use static_context::ContextRule;
pub use static_context::StaticContext;
pub(crate) use static_context::{FunctionType, StaticFunctionDescription};
