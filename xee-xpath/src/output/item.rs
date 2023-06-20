use xot::Xot;

use crate::error;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone)]
pub enum Item {
    StackItem(StackItem),
}

pub enum ItemValue {
    Atomic(output::Atomic),
    Function(output::Closure),
    Node(xml::Node),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackValue(pub(crate) stack::Value);
#[derive(Debug, Clone, PartialEq)]
pub struct StackItem(pub(crate) stack::Item);

impl Item {
    pub fn value(&self) -> ItemValue {
        match self {
            Item::StackItem(StackItem(i)) => match i {
                stack::Item::Atomic(a) => ItemValue::Atomic(output::Atomic::new(a.clone())),
                stack::Item::Function(f) => ItemValue::Function(f.to_output()),
                stack::Item::Node(n) => ItemValue::Node(*n),
            },
        }
    }

    pub fn to_atomic(&self) -> error::Result<output::Atomic> {
        Ok(match self {
            Item::StackItem(StackItem(i)) => output::Atomic::new(i.to_atomic()?),
            _ => return Err(error::Error::XPTY0004A),
        })
    }

    pub fn to_node(&self) -> error::Result<xml::Node> {
        Ok(match self {
            Item::StackItem(StackItem(i)) => i.to_node(),
        }?)
    }

    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        Ok(match self {
            Item::StackItem(StackItem(i)) => i.string_value(xot),
        }?)
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Item::StackItem(StackItem(i1)), Item::StackItem(StackItem(i2))) => i1 == i2,
        }
    }
}

impl From<stack::Item> for output::Item {
    fn from(stack_item: stack::Item) -> Self {
        Item::StackItem(StackItem(stack_item))
    }
}

impl From<xml::Node> for output::Item {
    fn from(node: xml::Node) -> Self {
        Item::StackItem(StackItem(stack::Item::Node(node)))
        // Item::StackValue(StackValue(stack::Value::Node(node)))
    }
}

impl From<output::Atomic> for output::Item {
    fn from(atomic: output::Atomic) -> Self {
        Item::StackItem(StackItem(stack::Item::Atomic(atomic.stack_atomic)))
    }
}

impl From<output::Item> for stack::Value {
    fn from(item: output::Item) -> Self {
        match item {
            Item::StackItem(StackItem(stack_item)) => stack_item.into_stack_value(),
        }
    }
}

impl From<output::Item> for stack::Item {
    fn from(item: output::Item) -> Self {
        match item {
            Item::StackItem(StackItem(i)) => i,
        }
    }
}

impl From<&output::Item> for stack::Item {
    fn from(item: &output::Item) -> Self {
        item.clone().into()
    }
}

impl TryFrom<&output::Item> for output::Atomic {
    type Error = error::Error;

    fn try_from(item: &output::Item) -> error::Result<Self> {
        item.to_atomic()
    }
}
