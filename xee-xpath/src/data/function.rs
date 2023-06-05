use miette::SourceSpan;

use crate::ast;
use crate::data::error::ValueError;
use crate::data::value::Value;
use crate::ir;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct FunctionId(pub(crate) usize);

impl FunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct StaticFunctionId(pub(crate) usize);

impl StaticFunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) constants: Vec<Value>,
    pub(crate) closure_names: Vec<ir::Name>,
    pub(crate) chunk: Vec<u8>,
    pub(crate) spans: Vec<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClosureFunctionId {
    Static(StaticFunctionId),
    Dynamic(FunctionId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub(crate) function_id: ClosureFunctionId,
    pub(crate) values: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Step {
    pub(crate) axis: ast::Axis,
    pub(crate) node_test: ast::NodeTest,
}
