use ahash::{HashMap, HashMapExt};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use xot::Xot;

use crate::ast;
use crate::error::{Error, Result};
use crate::name::FN_NAMESPACE;
use crate::value::{Atomic, StackValue, StaticFunctionId};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum ParameterType {
    Integer,
    String,
    Sequence,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Parameter {
    name: String,
    type_: ParameterType,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum ContextRule {
    ItemFirst,
    ItemSecond,
    PositionFirst,
    SizeFirst,
}

pub(crate) struct StaticFunction {
    name: ast::Name,
    parameters: Vec<Parameter>,
    return_type: ParameterType,
    pub(crate) context_rule: Option<ContextRule>,
    func: fn(xot: &Xot, arguments: &[StackValue]) -> Result<StackValue>,
}

impl Debug for StaticFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticFunction")
            .field("name", &self.name)
            .field("parameters", &self.parameters)
            .field("context_rule", &self.context_rule)
            .field("return_type", &self.return_type)
            .finish()
    }
}

impl StaticFunction {
    pub(crate) fn invoke(
        &self,
        xot: &Xot,
        arguments: &[StackValue],
        closure_values: &[StackValue],
    ) -> Result<StackValue> {
        if arguments.len() != self.parameters.len() {
            return Err(Error::TypeError);
        }
        if let Some(context_rule) = &self.context_rule {
            match context_rule {
                ContextRule::ItemFirst | ContextRule::PositionFirst | ContextRule::SizeFirst => {
                    let mut new_arguments = vec![closure_values[0].clone()];
                    new_arguments.extend_from_slice(arguments);
                    (self.func)(xot, &new_arguments)
                }
                ContextRule::ItemSecond => {
                    let mut new_arguments = arguments.to_vec();
                    new_arguments.push(closure_values[0].clone());
                    (self.func)(xot, &new_arguments)
                }
            }
        } else {
            (self.func)(xot, arguments)
        }
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
        let by_index = vec![
            StaticFunction {
                name: ast::Name::new("my_function".to_string(), None),
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
                context_rule: None,
                func: bound_my_function,
            },
            StaticFunction {
                name: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string())),
                parameters: vec![],
                return_type: ParameterType::Integer,
                context_rule: Some(ContextRule::PositionFirst),
                func: bound_position,
            },
            StaticFunction {
                name: ast::Name::new("local-name".to_string(), Some(FN_NAMESPACE.to_string())),
                parameters: vec![],
                return_type: ParameterType::String,
                context_rule: Some(ContextRule::ItemFirst),
                func: local_name,
            },
            StaticFunction {
                name: ast::Name::new("namespace-uri".to_string(), Some(FN_NAMESPACE.to_string())),
                parameters: vec![],
                return_type: ParameterType::String,
                context_rule: Some(ContextRule::ItemFirst),
                func: namespace_uri,
            },
            StaticFunction {
                name: ast::Name::new("count".to_string(), Some(FN_NAMESPACE.to_string())),
                parameters: vec![{
                    Parameter {
                        name: "nodes".to_string(),
                        type_: ParameterType::Sequence,
                    }
                }],
                return_type: ParameterType::Integer,
                context_rule: None,
                func: count,
            },
        ];
        for (i, static_function) in by_index.iter().enumerate() {
            by_name.insert(
                (
                    static_function.name.clone(),
                    static_function.parameters.len() as u8,
                ),
                StaticFunctionId(i),
            );
        }
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

#[derive(Debug)]
pub(crate) struct StaticContext {
    pub(crate) functions: StaticFunctions,
}

impl StaticContext {
    pub(crate) fn new() -> Self {
        Self {
            functions: StaticFunctions::new(),
        }
    }
}

fn my_function(a: i64, b: i64) -> i64 {
    a + b
}

fn bound_my_function(_xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
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

fn bound_position(_xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
    // position should be the context value
    Ok(arguments[0].clone())
}

fn local_name(xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
    let a = arguments[0].as_node().ok_or(Error::TypeError)?;
    Ok(StackValue::Atomic(Atomic::String(Rc::new(
        a.local_name(xot),
    ))))
}

fn namespace_uri(xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
    let a = arguments[0].as_node().ok_or(Error::TypeError)?;
    Ok(StackValue::Atomic(Atomic::String(Rc::new(
        a.namespace_uri(xot),
    ))))
}

fn count(_xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
    let a = arguments[0].as_sequence().ok_or(Error::TypeError)?;
    let a = a.borrow();
    Ok(StackValue::Atomic(Atomic::Integer(a.items.len() as i64)))
}
