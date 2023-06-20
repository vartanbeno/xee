use xot::Xot;

use crate::error;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    stack_item: stack::Item,
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
        match &self.stack_item {
            stack::Item::Atomic(a) => ItemValue::Atomic(output::Atomic::new(a.clone())),
            stack::Item::Function(f) => ItemValue::Function(f.to_output()),
            stack::Item::Node(n) => ItemValue::Node(*n),
        }
    }

    pub fn to_atomic(&self) -> error::Result<output::Atomic> {
        Ok(output::Atomic::new(self.stack_item.to_atomic()?))
    }

    pub fn to_node(&self) -> error::Result<xml::Node> {
        Ok(self.stack_item.to_node()?)
    }

    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        Ok(self.stack_item.string_value(xot)?)
    }
}

impl From<stack::Item> for output::Item {
    fn from(stack_item: stack::Item) -> Self {
        Self { stack_item }
    }
}

impl From<xml::Node> for output::Item {
    fn from(node: xml::Node) -> Self {
        Self {
            stack_item: stack::Item::Node(node),
        }
    }
}

impl From<output::Atomic> for output::Item {
    fn from(atomic: output::Atomic) -> Self {
        Self {
            stack_item: stack::Item::Atomic(atomic.stack_atomic),
        }
    }
}

impl From<output::Item> for stack::Value {
    fn from(item: output::Item) -> Self {
        item.stack_item.into_stack_value()
    }
}

impl From<output::Item> for stack::Item {
    fn from(item: output::Item) -> Self {
        item.stack_item
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
