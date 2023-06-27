use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;

use crate::error;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sequence(Rc<RefCell<InnerSequence>>);

impl Sequence {
    pub(crate) fn new(sequence: InnerSequence) -> Self {
        Self(Rc::new(RefCell::new(sequence)))
    }
    pub(crate) fn empty() -> Self {
        Self::new(InnerSequence::empty())
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<InnerSequence> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<InnerSequence> {
        self.0.borrow_mut()
    }

    pub(crate) fn items(&self) -> SequenceIter {
        SequenceIter::new(self.clone())
    }
}

impl TryFrom<stack::Value> for stack::Sequence {
    type Error = error::Error;

    fn try_from(value: stack::Value) -> error::Result<Self> {
        value.to_sequence()
    }
}

impl TryFrom<&stack::Value> for stack::Sequence {
    type Error = error::Error;

    fn try_from(value: &stack::Value) -> error::Result<Self> {
        value.to_sequence()
    }
}

impl From<Vec<stack::Item>> for stack::Sequence {
    fn from(items: Vec<stack::Item>) -> Self {
        Self::new(InnerSequence::new(items))
    }
}

impl From<stack::Item> for stack::Sequence {
    fn from(item: stack::Item) -> Self {
        Self::new(InnerSequence::new(vec![item]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InnerSequence {
    pub(crate) items: Vec<stack::Item>,
}

impl InnerSequence {
    pub(crate) fn new(items: Vec<stack::Item>) -> Self {
        Self { items }
    }

    pub(crate) fn empty() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn singleton(&self) -> error::Result<&stack::Item> {
        if self.items.len() == 1 {
            Ok(&self.items[0])
        } else {
            Err(error::Error::Type)
        }
    }

    pub(crate) fn push_value(&mut self, value: stack::Value) {
        match value {
            stack::Value::Empty => {}
            stack::Value::Item(item) => self.items.push(item),
            stack::Value::Sequence(s) => self.extend(s),
            stack::Value::Absent => panic!("Don't know how to handle absent"),
        }
    }

    pub(crate) fn push(&mut self, item: &stack::Item) {
        self.items.push(item.clone());
    }

    pub(crate) fn extend(&mut self, other: Sequence) {
        for item in &other.borrow().items {
            self.push(item);
        }
    }

    pub(crate) fn concat(&self, other: &InnerSequence) -> InnerSequence {
        let mut items = self.items.clone();
        items.extend(other.items.clone());
        InnerSequence { items }
    }

    pub(crate) fn union(
        &self,
        other: &InnerSequence,
        annotations: &xml::Annotations,
    ) -> error::Result<InnerSequence> {
        let mut s = HashSet::new();
        for item in &self.items {
            let node = match item {
                stack::Item::Node(node) => *node,
                stack::Item::Atomic(..) => return Err(error::Error::Type),
                stack::Item::Function(..) => return Err(error::Error::Type),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                stack::Item::Node(node) => *node,
                stack::Item::Atomic(..) => return Err(error::Error::Type),
                stack::Item::Function(..) => return Err(error::Error::Type),
            };
            s.insert(node);
        }

        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));

        let items = nodes.into_iter().map(stack::Item::Node).collect::<Vec<_>>();
        Ok(InnerSequence { items })
    }
}

pub(crate) struct SequenceIter {
    sequence: Sequence,
    index: usize,
}

impl SequenceIter {
    pub(crate) fn new(sequence: Sequence) -> Self {
        Self { sequence, index: 0 }
    }
}

impl Iterator for SequenceIter {
    type Item = stack::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let inner_sequence = self.sequence.borrow();
        if self.index < inner_sequence.len() {
            let item = inner_sequence.items[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
