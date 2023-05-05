use ahash::{HashMap, HashMapExt};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::ast;
use crate::context::Context;
use crate::name::{Namespaces, FN_NAMESPACE};
use crate::value::ValueError;
use crate::value::{Atomic, Node, StackValue, StaticFunctionId};

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
    name: ast::Name,
    arity: usize,
    function_type: Option<FunctionType>,
    func: fn(context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError>,
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

fn static_function_descriptions() -> Vec<StaticFunctionDescription> {
    vec![
        StaticFunctionDescription {
            name: ast::Name::new("my_function".to_string(), None),
            arity: 2,
            function_type: None,
            func: bound_my_function,
        },
        StaticFunctionDescription {
            name: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: Some(FunctionType::Position),
            func: bound_position,
        },
        StaticFunctionDescription {
            name: ast::Name::new("last".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 0,
            function_type: Some(FunctionType::Size),
            func: bound_last,
        },
        StaticFunctionDescription {
            name: ast::Name::new("local-name".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: local_name,
        },
        StaticFunctionDescription {
            name: ast::Name::new("namespace-uri".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: namespace_uri,
        },
        StaticFunctionDescription {
            name: ast::Name::new("count".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: None,
            func: count,
        },
        StaticFunctionDescription {
            name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: root,
        },
        StaticFunctionDescription {
            name: ast::Name::new("string".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: string,
        },
        StaticFunctionDescription {
            name: ast::Name::new("string".to_string(), Some(FN_NAMESPACE.to_string())),
            arity: 1,
            function_type: Some(FunctionType::ItemFirst),
            func: string,
        },
    ]
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
    func: fn(context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError>,
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
        context: &Context,
        arguments: &[StackValue],
        closure_values: &[StackValue],
    ) -> Result<StackValue, ValueError> {
        if arguments.len() != self.arity {
            return Err(ValueError::TypeError);
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
pub(crate) struct StaticContext<'a> {
    pub(crate) namespaces: &'a Namespaces<'a>,
    pub(crate) functions: StaticFunctions,
}

impl<'a> StaticContext<'a> {
    pub(crate) fn new(namespaces: &'a Namespaces<'a>) -> Self {
        Self {
            namespaces,
            functions: StaticFunctions::new(),
        }
    }
}

fn my_function(a: i64, b: i64) -> i64 {
    a + b
}

fn bound_my_function(
    context: &Context,
    arguments: &[StackValue],
) -> Result<StackValue, ValueError> {
    let a = arguments[0].as_atomic(context)?.as_integer()?;
    let b = arguments[1].as_atomic(context)?.as_integer()?;
    Ok(StackValue::Atomic(Atomic::Integer(my_function(a, b))))
}

fn bound_position(_context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    // position should be the context value
    Ok(arguments[0].clone())
}

fn bound_last(_context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    // size should be the context value
    Ok(arguments[0].clone())
}

fn local_name(context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    let a = arguments[0].as_node()?;
    Ok(StackValue::Atomic(Atomic::String(Rc::new(
        a.local_name(context.xot),
    ))))
}

fn namespace_uri(context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    let a = arguments[0].as_node()?;
    Ok(StackValue::Atomic(Atomic::String(Rc::new(
        a.namespace_uri(context.xot),
    ))))
}

fn count(_context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    let a = arguments[0].as_sequence()?;
    let a = a.borrow();
    Ok(StackValue::Atomic(Atomic::Integer(a.items.len() as i64)))
}

fn root(context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    let a = arguments[0].as_node()?;
    let xot_node = match a {
        Node::Xot(node) => node,
        Node::Attribute(node, _) => node,
        Node::Namespace(node, _) => node,
    };
    // XXX there should be a xot.root() to obtain this in one step
    let top = context.xot.top_element(xot_node);
    let root = context.xot.parent(top).unwrap();
    Ok(StackValue::Node(Node::Xot(root)))
}

fn string(context: &Context, arguments: &[StackValue]) -> Result<StackValue, ValueError> {
    Ok(StackValue::Atomic(Atomic::String(Rc::new(string_helper(
        context,
        &arguments[0],
    )?))))
}

fn string_helper(context: &Context, value: &StackValue) -> Result<String, ValueError> {
    let value = match value {
        StackValue::Atomic(atomic) => match atomic {
            Atomic::String(string) => string.to_string(),
            Atomic::Integer(integer) => integer.to_string(),
            Atomic::Float(float) => float.to_string(),
            Atomic::Boolean(boolean) => boolean.to_string(),
            Atomic::Double(double) => double.to_string(),
            Atomic::Empty => "".to_string(),
        },
        StackValue::Sequence(sequence) => {
            let sequence = sequence.borrow();
            let len = sequence.len();
            match len {
                0 => "".to_string(),
                1 => string_helper(context, &StackValue::from_item(sequence.items[0].clone()))?,
                _ => Err(ValueError::TypeError)?,
            }
        }
        StackValue::Node(node) => node.string(context.xot),
        StackValue::Closure(_) => Err(ValueError::TypeError)?,
        StackValue::Step(_) => Err(ValueError::TypeError)?,
    };
    Ok(value)
}
