use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    StackValue(StackValue),
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
    pub(crate) fn from_stack_item(stack_item: stack::Item) -> Self {
        Item::StackItem(StackItem(stack_item))
    }

    pub fn from_node(node: xml::Node) -> Self {
        Item::StackValue(StackValue(stack::Value::Node(node)))
    }

    pub fn from_atomic(atomic: output::Atomic) -> Self {
        Item::StackValue(StackValue(stack::Value::Atomic(atomic.stack_atomic)))
    }

    pub fn value(&self) -> ItemValue {
        match self {
            Item::StackValue(StackValue(v)) => match v {
                stack::Value::Atomic(a) => ItemValue::Atomic(output::Atomic::new(a.clone())),
                stack::Value::Sequence(s) => unreachable!("item can never be sequence"),
                stack::Value::Closure(f) => ItemValue::Function(f.to_output()),
                stack::Value::Step(s) => unreachable!(),
                stack::Value::Node(n) => ItemValue::Node(*n),
            },
            Item::StackItem(StackItem(i)) => match i {
                stack::Item::Atomic(a) => ItemValue::Atomic(output::Atomic::new(a.clone())),
                stack::Item::Function(f) => ItemValue::Function(f.to_output()),
                stack::Item::Node(n) => ItemValue::Node(*n),
            },
        }
    }

    pub(crate) fn into_stack_value(self) -> stack::Value {
        match self {
            Item::StackValue(StackValue(stack_value)) => stack_value,
            Item::StackItem(StackItem(stack_item)) => stack_item.into_stack_value(),
        }
    }

    pub(crate) fn to_stack_item(&self) -> stack::Item {
        match self {
            Item::StackValue(StackValue(v)) => match v {
                stack::Value::Atomic(a) => stack::Item::Atomic(a.clone()),
                stack::Value::Sequence(s) => unreachable!("item can never be sequence"),
                stack::Value::Closure(f) => stack::Item::Function(f.clone()),
                stack::Value::Step(s) => unreachable!(),
                stack::Value::Node(n) => stack::Item::Node(*n),
            },

            Item::StackItem(StackItem(i)) => i.clone(),
        }
    }

    // TODO these should not return ValueResult as they're in the public API
    pub fn to_atomic(&self) -> stack::Result<output::Atomic> {
        Ok(match self {
            Item::StackValue(StackValue(v)) => output::Atomic::new((v).to_atomic()?),
            Item::StackItem(StackItem(i)) => output::Atomic::new(i.to_atomic()?),
        })
    }

    pub fn to_node(&self) -> stack::Result<xml::Node> {
        match self {
            Item::StackValue(StackValue(v)) => v.to_node(),
            Item::StackItem(StackItem(i)) => i.to_node(),
        }
    }

    pub fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        match self {
            Item::StackValue(StackValue(v)) => v.string_value(xot),
            Item::StackItem(StackItem(i)) => i.string_value(xot),
        }
    }
}
