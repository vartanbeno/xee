use miette::SourceSpan;

use crate::ir;
use crate::stack;

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
    pub(crate) constants: Vec<stack::StackValue>,
    pub(crate) closure_names: Vec<ir::Name>,
    pub(crate) chunk: Vec<u8>,
    pub(crate) spans: Vec<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ClosureFunctionId {
    Static(StaticFunctionId),
    Dynamic(FunctionId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub(crate) function_id: ClosureFunctionId,
    pub(crate) values: Vec<stack::StackValue>,
}

impl Closure {
    pub(crate) fn to_output(&self) -> OutputClosure {
        OutputClosure {
            function_id: self.function_id,
            // values: self.values.iter().map(|v| v.to_output()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputClosure {
    pub(crate) function_id: ClosureFunctionId,
    // pub(crate) values: Vec<Vec<OutputItem>>,
}
