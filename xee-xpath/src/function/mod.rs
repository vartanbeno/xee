mod array;
mod function_core;
mod inline_function;
mod map;
mod program;
mod signature;
mod static_function;

pub(crate) use array::Array;
pub(crate) use function_core::{Function, InlineFunctionId, StaticFunctionId};
pub(crate) use inline_function::{CastType, InlineFunction};
pub(crate) use map::Map;
pub(crate) use program::Program;
pub(crate) use signature::Signature;

// we allow StaticFunctionType as it's used in the xpath_fn macro
#[allow(unused_imports)]
pub(crate) use static_function::StaticFunctionType;
pub(crate) use static_function::{FunctionKind, FunctionRule, StaticFunctionDescription};
pub(crate) use static_function::{StaticFunction, StaticFunctions};
