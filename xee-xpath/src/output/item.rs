use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Atomic(output::Atomic),
    Function(output::Closure),
    Node(xml::Node),
}

impl From<&Item> for stack::Item {
    fn from(item: &Item) -> Self {
        match item {
            Item::Atomic(a) => stack::Item::Atomic(a.into()),
            Item::Function(_f) => todo!("Cannot turn output functions into functions yet"),
            Item::Node(n) => stack::Item::Node(*n),
        }
    }
}

impl From<Item> for stack::Item {
    fn from(item: Item) -> Self {
        (&item).into()
    }
}

impl Item {
    // TODO these should not return ValueResult as they're in the public API
    pub fn to_atomic(&self) -> stack::Result<&output::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a),
            _ => Err(stack::Error::Type),
        }
    }
    pub fn to_node(&self) -> stack::Result<xml::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(stack::Error::Type),
        }
    }
    pub fn to_bool(&self) -> stack::Result<bool> {
        match self {
            Item::Atomic(a) => a.to_bool(),
            _ => Err(stack::Error::Type),
        }
    }
    pub fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        match self {
            Item::Atomic(a) => Ok(a.string_value()?),
            Item::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::Error::Type),
        }
    }
}
