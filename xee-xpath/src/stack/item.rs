use std::rc::Rc;
use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Atomic(stack::Atomic),
    // XXX what about static function references?
    Function(Rc<stack::Closure>),
    Node(xml::Node),
}

impl Item {
    pub(crate) fn to_output(&self) -> output::Item {
        match self {
            Item::Atomic(a) => output::Item::Atomic(a.to_output()),
            Item::Function(f) => output::Item::Function(f.to_output()),
            Item::Node(n) => output::Item::Node(*n),
        }
    }

    pub(crate) fn to_atomic(&self) -> stack::Result<&stack::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a),
            _ => Err(stack::Error::Type),
        }
    }
    pub(crate) fn to_node(&self) -> stack::Result<xml::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(stack::Error::Type),
        }
    }
    pub(crate) fn to_bool(&self) -> stack::Result<bool> {
        match self {
            Item::Atomic(a) => a.to_bool(),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        match self {
            Item::Atomic(a) => Ok(a.string_value()?),
            Item::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn into_stack_value(self) -> stack::Value {
        match self {
            Item::Atomic(a) => stack::Value::Atomic(a),
            Item::Node(n) => stack::Value::Node(n),
            Item::Function(f) => stack::Value::Closure(f),
        }
    }
}
