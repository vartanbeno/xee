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
            Item::Atomic(a) => a.effective_boolean_value(),
            // If its operand is a sequence whose first item is a node, fn:boolean returns true;
            // this is the case when a single node is on the stack, just like if it
            // were in a sequence.
            Item::Node(_) => Ok(true),
            // XXX the type error that the effective boolean wants is
            // NOT the normal type error, but err:FORG0006. We don't
            // make that distinction yet
            Item::Function(_) => Err(error::Error::Type),
        }
    }

    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        match self {
            Item::Atomic(atomic) => atomic.string_value(),
            Item::Node(node) => Ok(node.string_value(xot)),
            Item::Function(_) => Err(error::Error::Type),
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

#[derive(Debug, Clone)]
pub enum AtomizedItemIter {
    Atomic(std::iter::Once<atomic::Atomic>),
    Node(AtomizedNodeIter),
    // TODO: properly handle functions; for now they error
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl AtomizedItemIter {
    pub(crate) fn new(item: Item, xot: &Xot) -> Self {
        match item {
            Item::Atomic(a) => Self::Atomic(std::iter::once(a)),
            Item::Node(n) => Self::Node(AtomizedNodeIter::new(n, xot)),
            Item::Function(_) => Self::Erroring(std::iter::once(Err(error::Error::Type))),
        }
    }
}

impl Iterator for AtomizedItemIter {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atomic(iter) => iter.next().map(Ok),
            Self::Node(iter) => iter.next().map(Ok),
            Self::Erroring(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomizedNodeIter {
    typed_value: Vec<atomic::Atomic>,
    typed_value_index: usize,
}

impl AtomizedNodeIter {
    fn new(node: xml::Node, xot: &Xot) -> Self {
        Self {
            typed_value: node.typed_value(xot),
            typed_value_index: 0,
        }
    }
}

impl Iterator for AtomizedNodeIter {
    type Item = atomic::Atomic;

    fn next(&mut self) -> Option<Self::Item> {
        if self.typed_value_index < self.typed_value.len() {
            let item = self.typed_value[self.typed_value_index].clone();
            self.typed_value_index += 1;
            Some(item)
        } else {
            None
        }
    }
}
