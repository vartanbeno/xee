use std::rc::Rc;
use xot::Xot;

use crate::atomic;
use crate::error;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Atomic(atomic::Atomic),
    Function(Rc<stack::Closure>),
    Node(xml::Node),
}

impl Item {
    pub fn to_atomic(&self) -> error::Result<atomic::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(error::Error::Type),
        }
    }
    pub fn to_node(&self) -> error::Result<xml::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(error::Error::Type),
        }
    }

    pub fn to_function(&self) -> error::Result<&stack::Closure> {
        match self {
            Item::Function(f) => Ok(f.as_ref()),
            _ => Err(error::Error::Type),
        }
    }

    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            stack::Item::Atomic(a) => a.effective_boolean_value(),
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            stack::Item::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            stack::Item::Function(_) => Err(error::Error::Type),
        }
    }

    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        match self {
            stack::Item::Atomic(atomic) => atomic.string_value(),
            stack::Item::Node(node) => Ok(node.string_value(xot)),
            stack::Item::Function(_) => Err(error::Error::Type),
        }
    }
}

impl<T> From<T> for Item
where
    T: Into<atomic::Atomic>,
{
    fn from(a: T) -> Self {
        Self::Atomic(a.into())
    }
}

impl From<xml::Node> for Item {
    fn from(node: xml::Node) -> Self {
        Self::Node(node)
    }
}

impl From<stack::Closure> for Item {
    fn from(f: stack::Closure) -> Self {
        Self::Function(Rc::new(f))
    }
}
