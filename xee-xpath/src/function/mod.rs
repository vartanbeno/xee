mod array;
mod function_core;
mod map;
mod static_function;

pub(crate) use array::Array;
pub(crate) use function_core::{
    CastType, Closure, InlineFunction, InlineFunctionId, StaticFunctionId,
};
pub(crate) use map::Map;

// we allow StaticFunctionType as it's used in the xpath_fn macro
#[allow(unused_imports)]
pub(crate) use static_function::StaticFunctionType;
pub(crate) use static_function::StaticFunctions;
pub(crate) use static_function::{FunctionKind, FunctionRule, StaticFunctionDescription};
