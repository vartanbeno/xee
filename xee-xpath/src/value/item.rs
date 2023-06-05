use std::rc::Rc;
use xot::Xot;

use crate::value::atomic::Atomic;
use crate::value::error::ValueError;
use crate::value::node::Node;
use crate::value::value::{Closure, Value};

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Atomic(Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(Node),
}

impl Item {
    pub fn to_atomic(&self) -> Result<&Atomic> {
        match self {
            Item::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_node(&self) -> Result<Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> Result<bool> {
        match self {
            Item::Atomic(a) => a.to_bool(),
            _ => Err(ValueError::Type),
        }
    }

    pub fn string_value(&self, xot: &Xot) -> Result<String> {
        match self {
            Item::Atomic(a) => Ok(a.string_value()?),
            Item::Node(n) => Ok(n.string_value(xot)),
            _ => Err(ValueError::Type),
        }
    }

    pub fn to_stack_value(self) -> Value {
        match self {
            Item::Atomic(a) => Value::Atomic(a),
            Item::Node(n) => Value::Node(n),
            Item::Function(f) => Value::Closure(f),
        }
    }
}
