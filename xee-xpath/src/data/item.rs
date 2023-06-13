use std::rc::Rc;
use xot::Xot;

use super::atomic::{Atomic, OutputAtomic};
use super::error::ValueError;
use super::function::{Closure, OutputClosure};
use super::node::Node;
use super::value::Value;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Atomic(Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(Node),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OutputItem {
    Atomic(OutputAtomic),
    Function(OutputClosure),
    Node(Node),
}

impl From<OutputItem> for Item {
    fn from(item: OutputItem) -> Self {
        match item {
            OutputItem::Atomic(a) => Item::Atomic(a.into()),
            OutputItem::Function(_f) => todo!("Cannot turn output functions into functions yet"),
            OutputItem::Node(n) => Item::Node(n),
        }
    }
}

impl Item {
    pub fn to_output(&self) -> OutputItem {
        match self {
            Item::Atomic(a) => OutputItem::Atomic(a.to_output()),
            Item::Function(f) => OutputItem::Function(f.to_output()),
            Item::Node(n) => OutputItem::Node(*n),
        }
    }

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

impl OutputItem {
    pub fn to_atomic(&self) -> Result<&OutputAtomic> {
        match self {
            OutputItem::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_node(&self) -> Result<Node> {
        match self {
            OutputItem::Node(n) => Ok(*n),
            _ => Err(ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> Result<bool> {
        match self {
            OutputItem::Atomic(a) => a.to_bool(),
            _ => Err(ValueError::Type),
        }
    }
    pub fn string_value(&self, xot: &Xot) -> Result<String> {
        match self {
            OutputItem::Atomic(a) => Ok(a.string_value()?),
            OutputItem::Node(n) => Ok(n.string_value(xot)),
            _ => Err(ValueError::Type),
        }
    }
}
