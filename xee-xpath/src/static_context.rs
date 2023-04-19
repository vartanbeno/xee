use ahash::{HashMap, HashMapExt};
use std::fmt::{Debug, Formatter};
use xot::Xot;

use crate::ast;
use crate::error::{Error, Result};
use crate::value::{Atomic, StackValue, StaticFunctionId};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum ParameterType {
    Integer,
    String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Parameter {
    name: String,
    type_: ParameterType,
}

pub(crate) struct StaticFunction {
    parameters: Vec<Parameter>,
    return_type: ParameterType,
    func: fn(arguments: &[StackValue]) -> Result<StackValue>,
}

impl Debug for StaticFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticFunction")
            .field("parameters", &self.parameters)
            .field("return_type", &self.return_type)
            .finish()
    }
}

impl StaticFunction {
    pub(crate) fn invoke(&self, arguments: &[StackValue]) -> Result<StackValue> {
        if arguments.len() != self.parameters.len() {
            return Err(Error::TypeError);
        }
        (self.func)(arguments)
    }
}

#[derive(Debug)]
pub(crate) struct StaticFunctions {
    by_name: HashMap<(ast::Name, u8), StaticFunctionId>,
    by_index: Vec<StaticFunction>,
}

impl StaticFunctions {
    pub(crate) fn new() -> Self {
        let mut by_name = HashMap::new();
        let mut by_index = Vec::new();
        by_index.push(StaticFunction {
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    type_: ParameterType::Integer,
                },
                Parameter {
                    name: "b".to_string(),
                    type_: ParameterType::Integer,
                },
            ],
            return_type: ParameterType::Integer,
            func: bound_my_function,
        });
        by_name.insert(
            (
                ast::Name::new("my_function".to_string(), None),
                by_index[0].parameters.len() as u8,
            ),
            StaticFunctionId(0),
        );
        Self { by_name, by_index }
    }

    pub(crate) fn get_by_name(&self, name: &ast::Name, arity: u8) -> Option<StaticFunctionId> {
        // XXX annoying clone
        self.by_name.get(&(name.clone(), arity)).copied()
    }

    pub(crate) fn get_by_index(&self, static_function_id: StaticFunctionId) -> &StaticFunction {
        &self.by_index[static_function_id.0]
    }
}

pub(crate) struct StaticContext<'a> {
    pub(crate) functions: StaticFunctions,
    pub(crate) xot: &'a Xot,
}

impl<'a> StaticContext<'a> {
    pub(crate) fn new(xot: &'a Xot) -> Self {
        Self {
            functions: StaticFunctions::new(),
            xot,
        }
    }
}

impl<'a> Debug for StaticContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticContext")
            .field("functions", &self.functions)
            .finish()
    }
}

fn my_function(a: i64, b: i64) -> i64 {
    a + b
}

fn bound_my_function(arguments: &[StackValue]) -> Result<StackValue> {
    let a = arguments[0]
        .as_atomic()
        .ok_or(Error::TypeError)?
        .as_integer()
        .ok_or(Error::TypeError)?;
    let b = arguments[1]
        .as_atomic()
        .ok_or(Error::TypeError)?
        .as_integer()
        .ok_or(Error::TypeError)?;
    Ok(StackValue::Atomic(Atomic::Integer(my_function(a, b))))
}
