mod comparison;
mod function;
mod value;

pub(crate) use function::Closure;
pub(crate) use function::{CastType, InlineFunction, InlineFunctionId, StaticFunctionId};
pub(crate) use value::{AtomizedIter, Value, ValueIter};
