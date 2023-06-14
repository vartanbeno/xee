use std::rc::Rc;
use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

// TODO: the use in the macro needs to keep this public, needs to be investigated
// further.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StackValue {
    Atomic(stack::Atomic),
    Sequence(stack::Sequence),
    Closure(Rc<stack::Closure>),
    // StaticFunction(StaticFunctionId),
    Step(Rc<xml::Step>),
    Node(xml::Node),
}

impl StackValue {
    pub(crate) fn from_item(item: stack::Item) -> Self {
        match item {
            stack::Item::Atomic(a) => StackValue::Atomic(a),
            stack::Item::Node(n) => StackValue::Node(n),
            stack::Item::Function(f) => StackValue::Closure(f),
        }
    }

    pub(crate) fn from_items(items: &[stack::Item]) -> Self {
        if items.is_empty() {
            StackValue::Atomic(stack::Atomic::Empty)
        } else if items.len() == 1 {
            StackValue::from_item(items[0].clone())
        } else {
            StackValue::Sequence(stack::Sequence::from_items(items))
        }
    }

    pub(crate) fn into_output_sequence(self) -> output::Sequence {
        let seq = self.to_many();
        seq.to_output()
    }

    pub(crate) fn to_one(&self) -> stack::Result<stack::Item> {
        match self {
            StackValue::Atomic(a) => Ok(stack::Item::Atomic(a.clone())),
            StackValue::Sequence(s) => s.to_one(),
            StackValue::Node(n) => Ok(stack::Item::Node(*n)),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_option(&self) -> stack::Result<Option<stack::Item>> {
        match self {
            StackValue::Atomic(a) => Ok(Some(stack::Item::Atomic(a.clone()))),
            StackValue::Sequence(s) => s.to_option(),
            StackValue::Node(n) => Ok(Some(stack::Item::Node(*n))),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_many(&self) -> stack::Sequence {
        match self {
            StackValue::Atomic(a) => stack::Sequence::from_atomic(a),
            StackValue::Sequence(s) => s.clone(),
            StackValue::Node(n) => stack::Sequence::from_node(*n),
            // TODO: we need to handle the function case here, but
            // we don't handle it yet
            _ => {
                dbg!("unhandled to_many value {:?}", self);
                stack::Sequence::empty()
            }
        }
    }

    pub(crate) fn effective_boolean_value(&self) -> stack::Result<bool> {
        match self {
            StackValue::Atomic(a) => a.to_bool(),
            StackValue::Sequence(s) => {
                let s = s.borrow();
                // If its operand is an empty sequence, fn:boolean returns false.
                if s.is_empty() {
                    return Ok(false);
                }
                // If its operand is a sequence whose first item is a node, fn:boolean returns true.
                if matches!(s.items[0], stack::Item::Node(_)) {
                    return Ok(true);
                }
                // If its operand is a singleton value
                let singleton = s.singleton()?;
                singleton.to_bool()
            }
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            StackValue::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            StackValue::Closure(_) => Err(stack::Error::Type),
            StackValue::Step(_) => Err(stack::Error::Type),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            StackValue::Sequence(s) => s.borrow().is_empty(),
            StackValue::Atomic(stack::Atomic::Empty) => true,
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        let value = match self {
            StackValue::Atomic(atomic) => atomic.string_value()?,
            StackValue::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => StackValue::from_item(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(stack::Error::Type)?,
                }
            }
            StackValue::Node(node) => node.string_value(xot),
            StackValue::Closure(_) => Err(stack::Error::Type)?,
            StackValue::Step(_) => Err(stack::Error::Type)?,
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
        let a = stack::Atomic::Integer(1);
        let b = stack::Atomic::Decimal(Decimal::from(1));
        assert_eq!(a, b);
    }
}
