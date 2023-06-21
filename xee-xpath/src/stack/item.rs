use std::rc::Rc;
use xot::Xot;

use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Atomic(stack::Atomic),
    // XXX what about static function references?
    Function(Rc<stack::Closure>),
    Node(xml::Node),
}

impl Item {
    pub(crate) fn to_atomic(&self) -> stack::Result<stack::Atomic> {
        match self {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(stack::Error::Type),
        }
    }
    pub(crate) fn to_node(&self) -> stack::Result<xml::Node> {
        match self {
            Item::Node(n) => Ok(*n),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn to_function(&self) -> stack::Result<&stack::Closure> {
        match self {
            Item::Function(f) => Ok(f.as_ref()),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn effective_boolean_value(&self) -> stack::Result<bool> {
        match self {
            Item::Atomic(a) => a.effective_boolean_value(),
            _ => Err(stack::Error::Type),
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        match self {
            Item::Atomic(a) => Ok(a.string_value()?),
            Item::Node(n) => Ok(n.string_value(xot)),
            _ => Err(stack::Error::Type),
        }
    }
}

impl<T> From<T> for Item
where
    T: Into<stack::Atomic>,
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
