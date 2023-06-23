use std::rc::Rc;
use xot::Xot;

use crate::atomic;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Item {
    Atomic(atomic::Atomic),
    Function(Rc<stack::Closure>),
    Node(xml::Node),
}

impl Item {
    pub(crate) fn items(&self) -> ItemIter {
        ItemIter::new(self.clone())
    }

    pub(crate) fn to_atomic(&self) -> stack::Result<atomic::Atomic> {
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
            stack::Item::Atomic(a) => a.effective_boolean_value(),
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            stack::Item::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            stack::Item::Function(_) => Err(stack::Error::Type),
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        match self {
            stack::Item::Atomic(atomic) => atomic.string_value(),
            stack::Item::Node(node) => Ok(node.string_value(xot)),
            stack::Item::Function(_) => Err(stack::Error::Type),
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

pub(crate) struct ItemIter {
    item: Item,
    done: bool,
}

impl ItemIter {
    pub(crate) fn new(item: Item) -> Self {
        Self { item, done: false }
    }
}

impl Iterator for ItemIter {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        self.done = true;
        Some(self.item.clone())
    }
}
