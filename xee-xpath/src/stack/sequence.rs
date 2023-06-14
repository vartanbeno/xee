use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec;
use xot::Xot;

use crate::annotation::Annotations;
use crate::context::DynamicContext;
use crate::data::Node;
use crate::data::OutputSequence;
use crate::data::ValueError;

use super::Atomic;
use super::StackItem;
use super::StackValue;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StackSequence(Rc<RefCell<StackInnerSequence>>);

impl StackSequence {
    pub(crate) fn new(sequence: StackInnerSequence) -> Self {
        Self(Rc::new(RefCell::new(sequence)))
    }
    pub(crate) fn empty() -> Self {
        Self::new(StackInnerSequence::new())
    }
    pub(crate) fn from_atomic(atomic: &Atomic) -> Self {
        Self::new(StackInnerSequence::from_atomic(atomic.clone()))
    }
    pub(crate) fn from_node(node: Node) -> Self {
        Self::new(StackInnerSequence::from_node(node))
    }
    pub(crate) fn from_vec(items: Vec<StackItem>) -> Self {
        Self::new(StackInnerSequence::from_vec(items))
    }
    pub(crate) fn from_items(items: &[StackItem]) -> Self {
        Self::new(StackInnerSequence::from_items(items))
    }

    pub(crate) fn from_item(item: StackItem) -> Self {
        Self::new(StackInnerSequence::from_item(item))
    }

    pub fn borrow(&self) -> std::cell::Ref<StackInnerSequence> {
        self.0.borrow()
    }
    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<StackInnerSequence> {
        self.0.borrow_mut()
    }

    pub(crate) fn to_output(&self) -> OutputSequence {
        let s = self.0.borrow();
        OutputSequence::new(s.items.iter().map(|i| i.to_output()).collect())
    }

    pub(crate) fn to_one(&self) -> Result<StackItem> {
        let s = self.0.borrow();
        if s.len() == 1 {
            Ok(s.items[0].clone())
        } else {
            Err(ValueError::Type)
        }
    }

    pub(crate) fn to_option(&self) -> Result<Option<StackItem>> {
        let s = self.0.borrow();
        if s.len() == 1 {
            Ok(Some(s.items[0].clone()))
        } else if s.is_empty() {
            Ok(None)
        } else {
            Err(ValueError::Type)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StackInnerSequence {
    pub(crate) items: Vec<StackItem>,
}

impl StackInnerSequence {
    pub(crate) fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn as_slice(&self) -> &[StackItem] {
        &self.items
    }

    pub(crate) fn from_items(items: &[StackItem]) -> Self {
        Self {
            items: items.to_vec(),
        }
    }

    pub(crate) fn from_vec(items: Vec<StackItem>) -> Self {
        Self { items }
    }

    pub(crate) fn from_item(item: StackItem) -> Self {
        Self { items: vec![item] }
    }

    pub(crate) fn from_atomic(atomic: Atomic) -> Self {
        if matches!(atomic, Atomic::Empty) {
            return Self::new();
        }
        Self {
            items: vec![StackItem::Atomic(atomic)],
        }
    }

    pub(crate) fn from_node(node: Node) -> Self {
        Self {
            items: vec![StackItem::Node(node)],
        }
    }

    pub(crate) fn singleton(&self) -> Result<&StackItem> {
        if self.items.len() == 1 {
            Ok(&self.items[0])
        } else {
            Err(ValueError::Type)
        }
    }

    pub(crate) fn push_value(&mut self, value: StackValue) {
        match value {
            StackValue::Atomic(a) => self.items.push(StackItem::Atomic(a)),
            StackValue::Closure(c) => self.items.push(StackItem::Function(c)),
            StackValue::Sequence(s) => self.extend(s),
            StackValue::Node(n) => self.items.push(StackItem::Node(n)),
            _ => panic!("unexpected value: {:?}", value),
        }
    }

    pub(crate) fn push(&mut self, item: &StackItem) {
        self.items.push(item.clone());
    }

    pub(crate) fn extend(&mut self, other: StackSequence) {
        for item in &other.borrow().items {
            self.push(item);
        }
    }

    pub(crate) fn atomize(&self, xot: &Xot) -> StackInnerSequence {
        let mut items = Vec::new();
        for item in &self.items {
            match item {
                StackItem::Atomic(a) => items.push(StackItem::Atomic(a.clone())),
                StackItem::Node(n) => {
                    for typed_value in n.typed_value(xot) {
                        items.push(StackItem::Atomic(typed_value));
                    }
                }
                // XXX need code to handle array case
                StackItem::Function(..) => panic!("cannot atomize a function"),
            }
        }
        StackInnerSequence { items }
    }

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> Result<Atomic> {
        // we avoid using atomize as an optimization, so we don't
        // have to atomize all entries just to get the first item
        match self.len() {
            0 => Ok(Atomic::Empty),
            1 => {
                let item = &self.items[0];
                match item {
                    StackItem::Atomic(a) => Ok(a.clone()),
                    StackItem::Node(n) => {
                        let mut t = n.typed_value(context.xot);
                        if t.len() != 1 {
                            return Err(ValueError::XPTY0004);
                        }
                        Ok(t.remove(0))
                    }
                    // XXX need code to handle array case
                    StackItem::Function(..) => panic!("cannot atomize a function"),
                }
            }
            _ => Err(ValueError::XPTY0004),
        }
    }

    pub(crate) fn to_atoms(&self, xot: &Xot) -> Vec<Atomic> {
        let mut atoms = Vec::new();
        let atomized = self.atomize(xot);
        for item in atomized.items {
            match item {
                StackItem::Atomic(a) => atoms.push(a),
                _ => unreachable!("atomize returned a non-atomic item"),
            }
        }
        atoms
    }

    pub(crate) fn concat(&self, other: &StackInnerSequence) -> StackInnerSequence {
        let mut items = self.items.clone();
        items.extend(other.items.clone());
        StackInnerSequence { items }
    }

    pub(crate) fn union(
        &self,
        other: &StackInnerSequence,
        annotations: &Annotations,
    ) -> Result<StackInnerSequence> {
        let mut s = HashSet::new();
        for item in &self.items {
            let node = match item {
                StackItem::Node(node) => *node,
                StackItem::Atomic(..) => return Err(ValueError::Type),
                StackItem::Function(..) => return Err(ValueError::Type),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                StackItem::Node(node) => *node,
                StackItem::Atomic(..) => return Err(ValueError::Type),
                StackItem::Function(..) => return Err(ValueError::Type),
            };
            s.insert(node);
        }

        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));

        let items = nodes.into_iter().map(StackItem::Node).collect::<Vec<_>>();
        Ok(StackInnerSequence { items })
    }
}
