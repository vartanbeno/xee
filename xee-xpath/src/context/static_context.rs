use ahash::{HashMap, HashMapExt};
use std::fmt::{Debug, Formatter};

use crate::ast;
use crate::context::dynamic_context::DynamicContext;
use crate::context::namespaces::Namespaces;
use crate::context::static_functions::static_function_descriptions;
use crate::data::ValueError;
use crate::data::{StaticFunctionId, Value};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum FunctionType {
    // generate a function with one less arity that takes the
    // item as the first argument
    ItemFirst,
    // generate a function with one less arity that takes the item
    // as the last argument
    ItemLast,
    // this function takes position as the implicit only argument
    Position,
    // this function takes size as the implicit only argument
    Size,
}

pub(crate) struct StaticFunctionDescription {
    pub(crate) name: ast::Name,
    pub(crate) arity: usize,
    pub(crate) function_type: Option<FunctionType>,
    pub(crate) func: fn(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError>,
}

impl StaticFunctionDescription {
    fn functions(&self) -> Vec<StaticFunction> {
        if let Some(function_type) = &self.function_type {
            match function_type {
                FunctionType::ItemFirst => {
                    vec![
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity - 1,
                            context_rule: Some(ContextRule::ItemFirst),
                            func: self.func,
                        },
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity,
                            context_rule: None,
                            func: self.func,
                        },
                    ]
                }
                FunctionType::ItemLast => {
                    vec![
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity - 1,
                            context_rule: Some(ContextRule::ItemLast),
                            func: self.func,
                        },
                        StaticFunction {
                            name: self.name.clone(),
                            arity: self.arity,
                            context_rule: None,
                            func: self.func,
                        },
                    ]
                }
                FunctionType::Position => {
                    vec![StaticFunction {
                        name: self.name.clone(),
                        arity: self.arity,
                        context_rule: Some(ContextRule::PositionFirst),
                        func: self.func,
                    }]
                }
                FunctionType::Size => {
                    vec![StaticFunction {
                        name: self.name.clone(),
                        arity: self.arity,
                        context_rule: Some(ContextRule::SizeFirst),
                        func: self.func,
                    }]
                }
            }
        } else {
            vec![StaticFunction {
                name: self.name.clone(),
                arity: self.arity,
                context_rule: None,
                func: self.func,
            }]
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum ContextRule {
    ItemFirst,
    ItemLast,
    PositionFirst,
    SizeFirst,
}

pub(crate) struct StaticFunction {
    name: ast::Name,
    arity: usize,
    pub(crate) context_rule: Option<ContextRule>,
    func: fn(context: &DynamicContext, arguments: &[Value]) -> Result<Value, ValueError>,
}

impl Debug for StaticFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .field("context_rule", &self.context_rule)
            .finish()
    }
}

impl StaticFunction {
    pub(crate) fn invoke(
        &self,
        context: &DynamicContext,
        arguments: &[Value],
        closure_values: &[Value],
    ) -> Result<Value, ValueError> {
        if arguments.len() != self.arity {
            return Err(ValueError::Type);
        }
        if let Some(context_rule) = &self.context_rule {
            match context_rule {
                ContextRule::ItemFirst | ContextRule::PositionFirst | ContextRule::SizeFirst => {
                    let mut new_arguments = vec![closure_values[0].clone()];
                    new_arguments.extend_from_slice(arguments);
                    (self.func)(context, &new_arguments)
                }
                ContextRule::ItemLast => {
                    let mut new_arguments = arguments.to_vec();
                    new_arguments.push(closure_values[0].clone());
                    (self.func)(context, &new_arguments)
                }
            }
        } else {
            (self.func)(context, arguments)
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
        let descriptions = static_function_descriptions();
        let mut by_index = Vec::new();
        for description in descriptions {
            by_index.extend(description.functions());
        }

        for (i, static_function) in by_index.iter().enumerate() {
            by_name.insert(
                (static_function.name.clone(), static_function.arity as u8),
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
pub struct StaticContext<'a> {
    pub(crate) namespaces: &'a Namespaces<'a>,
    // XXX need to add in type later
    pub(crate) variables: Vec<ast::Name>,
    pub(crate) functions: StaticFunctions,
}

impl<'a> StaticContext<'a> {
    pub fn new(namespaces: &'a Namespaces<'a>) -> Self {
        Self {
            namespaces,
            variables: Vec::new(),
            functions: StaticFunctions::new(),
        }
    }

    pub fn with_variable_names(namespaces: &'a Namespaces<'a>, variables: &[ast::Name]) -> Self {
        Self {
            namespaces,
            variables: variables.to_vec(),
            functions: StaticFunctions::new(),
        }
    }
}
