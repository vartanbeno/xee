use ahash::HashMap;
use std::rc::Rc;
use xot::Xot;

use crate::atomic;
use crate::error;
use crate::sequence;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Atomic(atomic::Atomic),
    Function(Rc<stack::Closure>),
    Node(xml::Node),
    Map(Rc<HashMap<atomic::MapKey, Rc<sequence::Sequence>>>),
    Array(Rc<Vec<Rc<sequence::Sequence>>>),
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

    pub fn to_function(&self) -> error::Result<Rc<stack::Closure>> {
        match self {
            Item::Function(f) => Ok(f.clone()),
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
            Item::Function(_) | Item::Map(_) | Item::Array(_) => Err(error::Error::Type),
        }
    }

    pub fn string_value(&self, xot: &Xot) -> error::Result<String> {
        match self {
            Item::Atomic(atomic) => atomic.string_value(),
            Item::Node(node) => Ok(node.string_value(xot)),
            Item::Function(_) | Item::Map(_) | Item::Array(_) => Err(error::Error::Type),
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

impl From<Rc<stack::Closure>> for Item {
    fn from(f: Rc<stack::Closure>) -> Self {
        Self::Function(f)
    }
}

#[derive(Clone)]
pub enum AtomizedItemIter<'a> {
    Atomic(std::iter::Once<atomic::Atomic>),
    Node(AtomizedNodeIter),
    Array(AtomizedArrayIter<'a>),
    // TODO: properly handle functions; for now they error
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl<'a> AtomizedItemIter<'a> {
    pub(crate) fn new(item: Item, xot: &'a Xot) -> Self {
        match item {
            Item::Atomic(a) => Self::Atomic(std::iter::once(a)),
            Item::Node(n) => Self::Node(AtomizedNodeIter::new(n, xot)),
            Item::Array(a) => Self::Array(AtomizedArrayIter::new(a, xot)),
            Item::Function(_) => Self::Erroring(std::iter::once(Err(error::Error::FOTY0013))),
            Item::Map(_) => Self::Erroring(std::iter::once(Err(error::Error::FOTY0013))),
        }
    }
}

impl<'a> Iterator for AtomizedItemIter<'a> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atomic(iter) => iter.next().map(Ok),
            Self::Node(iter) => iter.next().map(Ok),
            Self::Array(iter) => iter.next(),
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

#[derive(Clone)]
pub struct AtomizedArrayIter<'a> {
    xot: &'a Xot,
    array: Rc<Vec<Rc<sequence::Sequence>>>,
    array_index: usize,
    iter: Option<Box<stack::AtomizedIter<'a>>>,
}

impl<'a> AtomizedArrayIter<'a> {
    fn new(array: Rc<Vec<Rc<sequence::Sequence>>>, xot: &'a Xot) -> Self {
        Self {
            xot,
            array,
            array_index: 0,
            iter: None,
        }
    }
}

impl<'a> Iterator for AtomizedArrayIter<'a> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if there there are any more atoms in this array entry,
            // supply those
            if let Some(iter) = &mut self.iter {
                if let Some(item) = iter.next() {
                    return Some(item);
                } else {
                    self.iter = None;
                }
            }
            // if we're at the end of the array, we're done
            if self.array_index >= self.array.len() {
                return None;
            }
            let sequence = self.array[self.array_index].clone();
            self.array_index += 1;

            self.iter = Some(Box::new(sequence.atomized(self.xot)));
        }
    }
}
