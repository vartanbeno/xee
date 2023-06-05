use std::cell::RefCell;
use std::rc::Rc;
use xot::Xot;

use crate::context::DynamicContext;
use crate::value::atomic::Atomic;
use crate::value::error::ValueError;
use crate::value::function::{Closure, Step};
use crate::value::item::Item;
use crate::value::node::Node;
use crate::value::sequence::Sequence;

type Result<T> = std::result::Result<T, ValueError>;

// Speculation: A rc value would be a lot smaller, though at the
// cost of indirection. So I'm not sure it would be faster; we'd get
// faster stack operations but slower heap access and less cache locality.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Atomic(Atomic),
    Sequence(Rc<RefCell<Sequence>>),
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

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> Result<Atomic> {
        match self {
            Value::Atomic(a) => Ok(a.clone()),
            Value::Sequence(s) => s.borrow().to_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }

    pub(crate) fn to_bool(&self) -> Result<bool> {
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

    pub fn to_sequence(&self) -> Result<Rc<RefCell<Sequence>>> {
        match self {
            Value::Sequence(s) => Ok(s.clone()),
            Value::Atomic(a) => Ok(Rc::new(RefCell::new(Sequence::from_atomic(a.clone())))),
            Value::Node(a) => Ok(Rc::new(RefCell::new(Sequence::from_node(*a)))),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_closure(&self) -> Result<&Closure> {
        match self {
            Value::Closure(c) => Ok(c),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_step(&self) -> Result<Rc<Step>> {
        match self {
            Value::Step(s) => Ok(Rc::clone(s)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_node(&self) -> Result<Node> {
        match self {
            Value::Node(n) => Ok(*n),
            Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(ValueError::Type),
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
