use std::rc::Rc;
use xot::Xot;

use super::atomic::Atomic;
use super::error::ValueError;
use super::function::{Closure, Step};
use super::item::Item;
use super::node::Node;
use super::sequence::{OutputSequence, Sequence};

type Result<T> = std::result::Result<T, ValueError>;

// TODO: the use in the macro needs to keep this public, needs to be investigated
// further.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Atomic(Atomic),
    Sequence(Sequence),
    Closure(Rc<Closure>),
    // StaticFunction(StaticFunctionId),
    Step(Rc<Step>),
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

    pub(crate) fn from_items(items: &[Item]) -> Self {
        if items.is_empty() {
            Value::Atomic(Atomic::Empty)
        } else if items.len() == 1 {
            Value::from_item(items[0].clone())
        } else {
            Value::Sequence(Sequence::from_items(items))
        }
    }

    pub(crate) fn into_output_sequence(self) -> OutputSequence {
        let seq = self.to_many();
        seq.to_output()
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

    pub(crate) fn to_many(&self) -> Sequence {
        match self {
            Value::Atomic(a) => Sequence::from_atomic(a),
            Value::Sequence(s) => s.clone(),
            Value::Node(n) => Sequence::from_node(*n),
            _ => panic!("Not handled yet"),
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
