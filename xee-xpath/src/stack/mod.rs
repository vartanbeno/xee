mod comparison;
mod function;
mod value;

pub use function::Closure;
pub(crate) use function::{CastType, ClosureFunctionId, Function, FunctionId, StaticFunctionId};
pub(crate) use value::{AtomizedIter, Value, ValueIter};
