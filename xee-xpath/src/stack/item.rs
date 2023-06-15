use std::rc::Rc;
use xot::Xot;

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
    pub(crate) fn to_atomic(&self) -> stack::Result<stack::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(stack::Error::Type),
        }
    }
    pub(crate) fn to_node(&self) -> stack::Result<xml::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(stack::Error::Type),
        }
    }
    pub(crate) fn effective_boolean_value(&self) -> stack::Result<bool> {
        match self {
            Item::Atomic(a) => a.effective_boolean_value(),
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
