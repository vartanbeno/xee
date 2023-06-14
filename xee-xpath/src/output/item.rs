use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Atomic(output::Atomic),
    Function(output::OutputClosure),
    Node(xml::Node),
}

impl From<&Item> for stack::StackItem {
    fn from(item: &Item) -> Self {
        match item {
            Item::Atomic(a) => stack::StackItem::Atomic(a.into()),
            Item::Function(_f) => todo!("Cannot turn output functions into functions yet"),
            Item::Node(n) => stack::StackItem::Node(*n),
        }
    }
}

impl From<Item> for stack::StackItem {
    fn from(item: Item) -> Self {
        (&item).into()
    }
}

impl Item {
    // TODO these should not return ValueResult as they're in the public API
    pub fn to_atomic(&self) -> stack::ValueResult<&output::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn to_node(&self) -> stack::ValueResult<xml::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn to_bool(&self) -> stack::ValueResult<bool> {
        match self {
            Item::Atomic(a) => a.to_bool(),
            _ => Err(stack::ValueError::Type),
        }
    }
    pub fn string_value(&self, xot: &Xot) -> stack::ValueResult<String> {
        match self {
            Item::Atomic(a) => Ok(a.string_value()?),
            Item::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::ValueError::Type),
        }
    }
}
