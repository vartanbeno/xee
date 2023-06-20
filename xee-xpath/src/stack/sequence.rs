use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec;

use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sequence(Rc<RefCell<InnerSequence>>);

impl Sequence {
    pub(crate) fn new(sequence: InnerSequence) -> Self {
        Self(Rc::new(RefCell::new(sequence)))
    }
    pub(crate) fn empty() -> Self {
        Self::new(InnerSequence::new())
    }
    pub(crate) fn from_atomic(atomic: &stack::Atomic) -> Self {
        Self::new(InnerSequence::from_atomic(atomic.clone()))
    }
    pub(crate) fn from_node(node: xml::Node) -> Self {
        Self::new(InnerSequence::from_node(node))
    }
    pub(crate) fn from_vec(items: Vec<stack::Item>) -> Self {
        Self::new(InnerSequence::from_vec(items))
    }
    pub(crate) fn from_items(items: &[stack::Item]) -> Self {
        Self::new(InnerSequence::from_items(items))
    }
    pub(crate) fn from_item(item: stack::Item) -> Self {
        Self::new(InnerSequence::from_item(item))
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
}

impl TryFrom<stack::Value> for stack::Sequence {
    type Error = stack::Error;

    fn try_from(value: stack::Value) -> stack::Result<Self> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&stack::Value> for stack::Sequence {
    type Error = stack::Error;

    fn try_from(value: &stack::Value) -> stack::Result<Self> {
        match value {
            stack::Value::Sequence(s) => Ok(s.clone()),
            stack::Value::Atomic(a) => Ok(stack::Sequence::from_atomic(a)),
            stack::Value::Node(n) => Ok(stack::Sequence::from_node(*n)),
            _ => Err(stack::Error::Type),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InnerSequence {
    pub(crate) items: Vec<stack::Item>,
}

impl InnerSequence {
    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn as_slice(&self) -> &[stack::Item] {
        &self.items
    }

    pub(crate) fn from_items(items: &[stack::Item]) -> Self {
        Self {
            items: items.to_vec(),
        }
    }

    pub(crate) fn from_vec(items: Vec<stack::Item>) -> Self {
        Self { items }
    }

    pub(crate) fn from_item(item: stack::Item) -> Self {
        Self { items: vec![item] }
    }

    pub(crate) fn from_atomic(atomic: stack::Atomic) -> Self {
        if matches!(atomic, stack::Atomic::Empty) {
            return Self::new();
        }
        Self {
            items: vec![stack::Item::Atomic(atomic)],
        }
    }

    pub(crate) fn from_node(node: xml::Node) -> Self {
        Self {
            items: vec![stack::Item::Node(node)],
        }
    }

    pub(crate) fn singleton(&self) -> stack::Result<&stack::Item> {
        if self.items.len() == 1 {
            Ok(&self.items[0])
        } else {
            Err(stack::Error::Type)
        }
    }

    pub(crate) fn push_value(&mut self, value: stack::Value) {
        match value {
            stack::Value::Atomic(a) => self.items.push(stack::Item::Atomic(a)),
            stack::Value::Closure(c) => self.items.push(stack::Item::Function(c)),
            stack::Value::Sequence(s) => self.extend(s),
            stack::Value::Node(n) => self.items.push(stack::Item::Node(n)),
            _ => panic!("unexpected value: {:?}", value),
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
    ) -> stack::Result<InnerSequence> {
        let mut s = HashSet::new();
        for item in &self.items {
            let node = match item {
                stack::Item::Node(node) => *node,
                stack::Item::Atomic(..) => return Err(stack::Error::Type),
                stack::Item::Function(..) => return Err(stack::Error::Type),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                stack::Item::Node(node) => *node,
                stack::Item::Atomic(..) => return Err(stack::Error::Type),
                stack::Item::Function(..) => return Err(stack::Error::Type),
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
