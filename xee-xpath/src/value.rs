use crate::ast;
use crate::error::{Error, Result};
use crate::instruction::{decode_instructions, Instruction};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct FunctionId(pub(crate) usize);

impl FunctionId {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0 as u16
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) constants: Vec<StackValue>,
    pub(crate) closure_names: Vec<ast::Name>,
    pub(crate) chunk: Vec<u8>,
}

impl Function {
    pub(crate) fn decoded(&self) -> Vec<Instruction> {
        decode_instructions(&self.chunk)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Closure {
    pub(crate) function_id: FunctionId,
    pub(crate) values: Vec<StackValue>,
}

// TODO: could we shrink this by pointing to a value heap with a reference
// smaller than 64 bits?
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StackValue {
    Integer(i64),
    Closure(Closure),
}

impl StackValue {
    pub(crate) fn as_integer(&self) -> Result<i64> {
        match self {
            StackValue::Integer(i) => Ok(*i),
            _ => Err(Error::TypeError),
        }
    }

    pub(crate) fn as_bool(&self) -> Result<bool> {
        match self {
            StackValue::Integer(i) => Ok(*i != 0),
            _ => Err(Error::TypeError),
        }
    }

    pub(crate) fn as_closure(&self) -> Result<&Closure> {
        match self {
            StackValue::Closure(c) => Ok(c),
            _ => Err(Error::TypeError),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub(crate) struct Sequence {
//     pub(crate) items: Vec<Item>,
// }
