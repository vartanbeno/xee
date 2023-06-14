use std::rc::Rc;
use xot::Xot;

use crate::data::Closure;
use crate::data::OutputItem;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackItem {
    Atomic(stack::Atomic),
    // XXX what about static function references?
    Function(Rc<Closure>),
    Node(xml::Node),
}

impl StackItem {
    pub(crate) fn to_output(&self) -> OutputItem {
        match self {
            StackItem::Atomic(a) => OutputItem::Atomic(a.to_output()),
            StackItem::Function(f) => OutputItem::Function(f.to_output()),
            StackItem::Node(n) => OutputItem::Node(*n),
        }
    }

    pub(crate) fn to_atomic(&self) -> stack::ValueResult<&stack::Atomic> {
        match self {
            StackItem::Atomic(a) => Ok(a),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub(crate) fn to_node(&self) -> stack::ValueResult<xml::Node> {
        match self {
            StackItem::Node(n) => Ok(*n),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub(crate) fn to_bool(&self) -> stack::ValueResult<bool> {
        match self {
            StackItem::Atomic(a) => a.to_bool(),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::ValueResult<String> {
        match self {
            StackItem::Atomic(a) => Ok(a.string_value()?),
            StackItem::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub(crate) fn into_stack_value(self) -> stack::StackValue {
        match self {
            StackItem::Atomic(a) => stack::StackValue::Atomic(a),
            StackItem::Node(n) => stack::StackValue::Node(n),
            StackItem::Function(f) => stack::StackValue::Closure(f),
        }
    }
}
