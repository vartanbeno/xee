mod dynamic_context;
mod namespaces;
mod static_context;
mod static_functions;

pub use dynamic_context::DynamicContext;
pub use namespaces::Namespaces;
pub(crate) use namespaces::FN_NAMESPACE;
pub(crate) use static_context::ContextRule;
pub use static_context::StaticContext;
