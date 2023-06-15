use std::rc::Rc;
use xot::Xot;

use crate::output;
use crate::stack;
use crate::xml;

// TODO: the use in the macro needs to keep this public, needs to be investigated
// further.
#[derive(Debug, Clone)]
pub(crate) enum Value {
    Atomic(stack::Atomic),
    Sequence(stack::Sequence),
    Closure(Rc<stack::Closure>),
    // StaticFunction(StaticFunctionId),
    Step(Rc<xml::Step>),
    Node(xml::Node),
}

impl Value {
    pub(crate) fn from_item(item: stack::Item) -> Self {
        match item {
            stack::Item::Atomic(a) => Value::Atomic(a),
            stack::Item::Node(n) => Value::Node(n),
            stack::Item::Function(f) => Value::Closure(f),
        }
    }

    pub(crate) fn from_items(items: &[stack::Item]) -> Self {
        if items.is_empty() {
            Value::Atomic(stack::Atomic::Empty)
        } else if items.len() == 1 {
            Value::from_item(items[0].clone())
        } else {
            Value::Sequence(stack::Sequence::from_items(items))
        }
    }

    pub(crate) fn into_output(self) -> output::Sequence {
        output::Sequence::new(self)
    }

    pub(crate) fn to_node(&self) -> stack::Result<xml::Node> {
        match self {
            Value::Node(n) => Ok(*n),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_one(&self) -> stack::Result<stack::Item> {
        match self {
            Value::Atomic(a) => Ok(stack::Item::Atomic(a.clone())),
            Value::Sequence(s) => s.to_one(),
            Value::Node(n) => Ok(stack::Item::Node(*n)),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_option(&self) -> stack::Result<Option<stack::Item>> {
        match self {
            Value::Atomic(a) => Ok(Some(stack::Item::Atomic(a.clone()))),
            Value::Sequence(s) => s.to_option(),
            Value::Node(n) => Ok(Some(stack::Item::Node(*n))),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_many(&self) -> stack::Sequence {
        match self {
            Value::Atomic(a) => stack::Sequence::from_atomic(a),
            Value::Sequence(s) => s.clone(),
            Value::Node(n) => stack::Sequence::from_node(*n),
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
            Value::Atomic(a) => a.effective_boolean_value(),
            Value::Sequence(s) => {
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
                singleton.effective_boolean_value()
            }
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            Value::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            Value::Closure(_) => Err(stack::Error::Type),
            Value::Step(_) => Err(stack::Error::Type),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            Value::Sequence(s) => s.borrow().is_empty(),
            Value::Atomic(stack::Atomic::Empty) => true,
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        let value = match self {
            Value::Atomic(atomic) => atomic.string_value()?,
            Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => Value::from_item(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(stack::Error::Type)?,
                }
            }
            Value::Node(node) => node.string_value(xot),
            Value::Closure(_) => Err(stack::Error::Type)?,
            Value::Step(_) => Err(stack::Error::Type)?,
        };
        Ok(value)
    }

    pub(crate) fn atomized<'a>(&self, xot: &'a Xot) -> stack::AtomizedIter<'a> {
        stack::AtomizedIter::new(self.clone(), xot)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        // comparisons between values are tricky, as a value
        // may be a single item or a sequence of items.
        // If they are single items, the comparison is easy,
        // if one half is a sequence (or an empty atomic) and
        // the other half is not, then we convert the value into a sequence first
        // before comparing
        match (self, other) {
            (Value::Atomic(a), Value::Atomic(b)) => a == b,
            (Value::Sequence(a), Value::Sequence(b)) => a == b,
            (Value::Atomic(stack::Atomic::Empty), Value::Sequence(b)) => b.is_empty(),
            (Value::Sequence(a), Value::Atomic(stack::Atomic::Empty)) => a.is_empty(),
            (Value::Closure(a), Value::Closure(b)) => a == b,
            (Value::Step(a), Value::Step(b)) => a == b,
            (Value::Node(a), Value::Node(b)) => a == b,
            (_, Value::Sequence(b)) => (&self.to_many()) == b,
            (Value::Sequence(a), _) => a == &other.to_many(),
            _ => false,
        }
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
