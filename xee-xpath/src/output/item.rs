use xot::Xot;

use crate::error;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone)]
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
                stack::Value::Sequence(_) => unreachable!("item can never be sequence"),
                stack::Value::Closure(f) => ItemValue::Function(f.to_output()),
                stack::Value::Step(_) => unreachable!(),
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
                stack::Value::Sequence(_) => unreachable!("item can never be sequence"),
                stack::Value::Closure(f) => stack::Item::Function(f.clone()),
                stack::Value::Step(_) => unreachable!(),
                stack::Value::Node(n) => stack::Item::Node(*n),
            },

            Item::StackItem(StackItem(i)) => i.clone(),
        }
    }

    pub fn to_atomic(&self) -> error::Result<output::Atomic> {
        Ok(match self {
            // at this point we *either* refer to a single value, or a stack
            // item. The stack value can never be multiple values
            Item::StackValue(StackValue(stack::Value::Atomic(atomic))) => {
                output::Atomic::new(atomic.clone())
            }
            Item::StackItem(StackItem(i)) => output::Atomic::new(i.to_atomic()?),
            _ => return Err(error::Error::XPTY0004A),
        })
    }

    pub fn to_node(&self) -> error::Result<xml::Node> {
        Ok(match self {
            Item::StackValue(StackValue(v)) => v.to_node(),
            Item::StackItem(StackItem(i)) => i.to_node(),
        }?)
    }

    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        Ok(match self {
            Item::StackValue(StackValue(v)) => v.string_value(xot),
            Item::StackItem(StackItem(i)) => i.string_value(xot),
        }?)
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Item::StackValue(StackValue(v1)), Item::StackValue(StackValue(v2))) => v1 == v2,
            (Item::StackItem(StackItem(i1)), Item::StackItem(StackItem(i2))) => i1 == i2,
            (Item::StackValue(StackValue(v1)), Item::StackItem(StackItem(i2))) => {
                v1 == &i2.to_stack_value()
            }
            (Item::StackItem(StackItem(i1)), Item::StackValue(StackValue(v2))) => {
                i1.to_stack_value() == *v2
            }
        }
    }
}
