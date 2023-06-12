use std::rc::Rc;
use xot::Xot;

use super::atomic::{Atomic, OutputAtomic};
use super::error::ValueError;
use super::function::{Closure, Step};
use super::item::{Item, OutputItem};
use super::node::Node;
use super::sequence::Sequence;

type Result<T> = std::result::Result<T, ValueError>;

// Speculation: A rc value would be a lot smaller, though at the
// cost of indirection. So I'm not sure it would be faster; we'd get
// faster stack operations but slower heap access and less cache locality.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Atomic(Atomic),
    Sequence(Sequence),
    Closure(Rc<Closure>),
    // StaticFunction(StaticFunctionId),
    Step(Rc<Step>),
    Node(Node),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OutputValue {
    Atomic(OutputAtomic),
    Sequence(Vec<OutputItem>),
    Closure(Closure),
    Step(Step),
    Node(Node),
}

impl Value {
    pub(crate) fn from_item(item: Item) -> Self {
        match item {
            Item::Atomic(a) => Value::Atomic(a),
            Item::Node(n) => Value::Node(n),
            Item::Function(f) => Value::Closure(f),
        }
    }

    pub fn to_output(&self) -> OutputValue {
        match self {
            Value::Atomic(Atomic::Empty) => OutputValue::Sequence(vec![]),
            Value::Atomic(a) => OutputValue::Atomic(a.to_output()),
            Value::Sequence(s) => OutputValue::Sequence(s.to_output()),
            Value::Closure(f) => OutputValue::Closure(f.as_ref().clone()),
            Value::Step(s) => OutputValue::Step(s.as_ref().clone()),
            Value::Node(n) => OutputValue::Node(*n),
        }
    }

    pub(crate) fn to_one(&self) -> Result<Item> {
        match self {
            Value::Atomic(a) => Ok(Item::Atomic(a.clone())),
            Value::Sequence(s) => s.to_one(),
            Value::Node(n) => Ok(Item::Node(*n)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_option(&self) -> Result<Option<Item>> {
        match self {
            Value::Atomic(a) => Ok(Some(Item::Atomic(a.clone()))),
            Value::Sequence(s) => s.to_option(),
            Value::Node(n) => Ok(Some(Item::Node(*n))),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_many(&self) -> Result<Sequence> {
        match self {
            Value::Atomic(a) => Ok(Sequence::from_atomic(a)),
            Value::Sequence(s) => Ok(s.clone()),
            Value::Node(n) => Ok(Sequence::from_node(*n)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn effective_boolean_value(&self) -> Result<bool> {
        match self {
            Value::Atomic(a) => a.to_bool(),
            Value::Sequence(s) => {
                let s = s.borrow();
                // If its operand is an empty sequence, fn:boolean returns false.
                if s.is_empty() {
                    return Ok(false);
                }
                // If its operand is a sequence whose first item is a node, fn:boolean returns true.
                if matches!(s.items[0], Item::Node(_)) {
                    return Ok(true);
                }
                // If its operand is a singleton value
                let singleton = s.singleton()?;
                singleton.to_bool()
            }
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            Value::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            Value::Closure(_) => Err(ValueError::Type),
            Value::Step(_) => Err(ValueError::Type),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            Value::Sequence(s) => s.borrow().is_empty(),
            Value::Atomic(Atomic::Empty) => true,
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> Result<String> {
        let value = match self {
            Value::Atomic(atomic) => atomic.string_value()?,
            Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => Value::from_item(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(ValueError::Type)?,
                }
            }
            Value::Node(node) => node.string_value(xot),
            Value::Closure(_) => Err(ValueError::Type)?,
            Value::Step(_) => Err(ValueError::Type)?,
        };
        Ok(value)
    }
}

impl OutputValue {
    pub(crate) fn to_items(self) -> Vec<OutputItem> {
        match self {
            OutputValue::Atomic(a) => vec![OutputItem::Atomic(a)],
            OutputValue::Sequence(s) => s,
            OutputValue::Closure(_) => todo!("cannot convert closure to items yet"),
            OutputValue::Step(_) => panic!("cannot convert step to items"),
            OutputValue::Node(n) => vec![OutputItem::Node(n)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_integer_compares_with_decimal() {
        let a = Atomic::Integer(1);
        let b = Atomic::Decimal(Decimal::from(1));
        assert_eq!(a, b);
    }
}
