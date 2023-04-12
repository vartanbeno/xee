use crate::ast;
use crate::error::{Error, Result};
use crate::instruction::{decode_instructions, Instruction};
use crate::static_context::StaticFunction;

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
    StaticFunction(StaticFunctionId),
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

    pub(crate) fn as_static_function(&self) -> Option<StaticFunctionId> {
        match self {
            StackValue::StaticFunction(f) => Some(*f),
            _ => None,
        }
    }
}

// https://www.w3.org/TR/xpath-datamodel-31/#xs-types
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AtomicValue {
    String(String),
    Boolean(bool),
    // Decimal, use a decimal type
    Integer(i64), // is really a decimal, but special case it for now
    Float(f32),
    Double(f64),
    // and many more
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Atomic(AtomicValue),
    Function(Closure),
    // XXX or a Node
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sequence {
    pub(crate) items: Vec<Item>,
}

impl Sequence {
    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub(crate) fn singleton(&self) -> Option<&Item> {
        if self.items.len() == 1 {
            Some(&self.items[0])
        } else {
            None
        }
    }

    pub(crate) fn push(&mut self, item: Item) {
        self.items.push(item);
    }

    pub(crate) fn extend(&mut self, other: Sequence) {
        self.items.extend(other.items);
    }
}
