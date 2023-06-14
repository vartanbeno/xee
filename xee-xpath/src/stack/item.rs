use std::rc::Rc;
use xot::Xot;

use crate::data::Closure;
use crate::data::Node;
use crate::data::OutputItem;
use crate::data::ValueError;
use crate::stack::Atomic;
use crate::stack::StackValue;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackItem {
    Atomic(Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(Node),
}

impl StackItem {
    pub(crate) fn to_output(&self) -> OutputItem {
        match self {
            StackItem::Atomic(a) => OutputItem::Atomic(a.to_output()),
            StackItem::Function(f) => OutputItem::Function(f.to_output()),
            StackItem::Node(n) => OutputItem::Node(*n),
        }
    }

    pub(crate) fn to_atomic(&self) -> Result<&Atomic> {
        match self {
            StackItem::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
    pub(crate) fn to_node(&self) -> Result<Node> {
        match self {
            StackItem::Node(n) => Ok(*n),
            _ => Err(ValueError::Type),
        }
    }
    pub(crate) fn to_bool(&self) -> Result<bool> {
        match self {
            StackItem::Atomic(a) => a.to_bool(),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> Result<String> {
        match self {
            StackItem::Atomic(a) => Ok(a.string_value()?),
            StackItem::Node(n) => Ok(n.string_value(xot)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn into_stack_value(self) -> StackValue {
        match self {
            StackItem::Atomic(a) => StackValue::Atomic(a),
            StackItem::Node(n) => StackValue::Node(n),
            StackItem::Function(f) => StackValue::Closure(f),
        }
    }
}
