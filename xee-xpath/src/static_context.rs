use ahash::{HashMap, HashMapExt};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use xot::Xot;

use crate::ast;
use crate::error::{Error, Result};
use crate::name::{Namespaces, FN_NAMESPACE};
use crate::value::{Atomic, Node, StackValue, StaticFunctionId};

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
    func: fn(xot: &Xot, arguments: &[StackValue]) -> Result<StackValue>,
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
        xot: &Xot,
        arguments: &[StackValue],
        closure_values: &[StackValue],
    ) -> Result<StackValue> {
        if arguments.len() != self.arity {
            return Err(Error::TypeError);
        }
        if let Some(context_rule) = &self.context_rule {
            match context_rule {
                ContextRule::ItemFirst | ContextRule::PositionFirst | ContextRule::SizeFirst => {
                    let mut new_arguments = vec![closure_values[0].clone()];
                    new_arguments.extend_from_slice(arguments);
                    (self.func)(xot, &new_arguments)
                }
                ContextRule::ItemLast => {
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
                arity: 2,
                context_rule: None,
                func: bound_my_function,
            },
            StaticFunction {
                name: ast::Name::new("position".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 0,
                context_rule: Some(ContextRule::PositionFirst),
                func: bound_position,
            },
            StaticFunction {
                name: ast::Name::new("local-name".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 0,
                context_rule: Some(ContextRule::ItemFirst),
                func: local_name,
            },
            StaticFunction {
                name: ast::Name::new("namespace-uri".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 0,
                context_rule: Some(ContextRule::ItemFirst),
                func: namespace_uri,
            },
            StaticFunction {
                name: ast::Name::new("count".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 1,
                context_rule: None,
                func: count,
            },
            StaticFunction {
                name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 0,
                context_rule: Some(ContextRule::ItemFirst),
                func: root,
            },
            StaticFunction {
                name: ast::Name::new("root".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 1,
                context_rule: None,
                func: root,
            },
            StaticFunction {
                name: ast::Name::new("string".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 1,
                context_rule: None,
                func: string,
            },
            StaticFunction {
                name: ast::Name::new("string".to_string(), Some(FN_NAMESPACE.to_string())),
                arity: 0,
                context_rule: Some(ContextRule::ItemFirst),
                func: string,
            },
        ];
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

fn root(xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
    let a = arguments[0].as_node().ok_or(Error::TypeError)?;
    let xot_node = match a {
        Node::Xot(node) => node,
        Node::Attribute(node, _) => node,
        Node::Namespace(node, _) => node,
    };
    // XXX there should be a xot.root() to obtain this in one step
    let top = xot.top_element(xot_node);
    let root = xot.parent(top).unwrap();
    Ok(StackValue::Node(Node::Xot(root)))
}

fn string(xot: &Xot, arguments: &[StackValue]) -> Result<StackValue> {
    Ok(StackValue::Atomic(Atomic::String(Rc::new(string_helper(
        xot,
        &arguments[0],
    )?))))
}

fn string_helper(xot: &Xot, value: &StackValue) -> Result<String> {
    let value = match value {
        StackValue::Atomic(atomic) => match atomic {
            Atomic::String(string) => string.to_string(),
            Atomic::Integer(integer) => integer.to_string(),
            Atomic::Float(float) => float.to_string(),
            Atomic::Boolean(boolean) => boolean.to_string(),
            Atomic::Double(double) => double.to_string(),
        },
        StackValue::Sequence(sequence) => {
            let sequence = sequence.borrow();
            let len = sequence.len();
            match len {
                0 => "".to_string(),
                1 => string_helper(xot, &StackValue::from_item(sequence.items[0].clone()))?,
                _ => Err(Error::TypeError)?,
            }
        }
        StackValue::Node(node) => node.string(xot),
        StackValue::Closure(_) => Err(Error::TypeError)?,
        StackValue::Step(_) => Err(Error::TypeError)?,
    };
    Ok(value)
}
