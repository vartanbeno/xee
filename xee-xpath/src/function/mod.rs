mod array;
mod function_core;
mod map;
mod program;
mod static_function;

pub(crate) use array::Array;
pub(crate) use function_core::{
    CastType, Function, InlineFunction, InlineFunctionId, Signature, StaticFunctionId,
};
pub(crate) use map::Map;
pub(crate) use program::Program;

// we allow StaticFunctionType as it's used in the xpath_fn macro
#[allow(unused_imports)]
pub(crate) use static_function::StaticFunctionType;
pub(crate) use static_function::{FunctionKind, FunctionRule, StaticFunctionDescription};
pub(crate) use static_function::{StaticFunction, StaticFunctions};
