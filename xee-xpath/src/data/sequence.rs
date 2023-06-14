use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec;
use xot::Xot;

use crate::annotation::Annotations;
use crate::context::DynamicContext;

use super::atomic::Atomic;
use super::error::ValueError;
use super::item::{Item, OutputItem};
use super::node::Node;
use super::value::Value;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sequence(Rc<RefCell<InnerSequence>>);

impl Sequence {
    pub(crate) fn new(sequence: InnerSequence) -> Self {
        Self(Rc::new(RefCell::new(sequence)))
    }
    pub(crate) fn empty() -> Self {
        Self::new(InnerSequence::new())
    }
    pub(crate) fn from_atomic(atomic: &Atomic) -> Self {
        Self::new(InnerSequence::from_atomic(atomic.clone()))
    }
    pub(crate) fn from_node(node: Node) -> Self {
        Self::new(InnerSequence::from_node(node))
    }
    pub(crate) fn from_vec(items: Vec<Item>) -> Self {
        Self::new(InnerSequence::from_vec(items))
    }
    pub(crate) fn from_items(items: &[Item]) -> Self {
        Self::new(InnerSequence::from_items(items))
    }

    pub(crate) fn from_item(item: Item) -> Self {
        Self::new(InnerSequence::from_item(item))
    }

    pub fn borrow(&self) -> std::cell::Ref<InnerSequence> {
        self.0.borrow()
    }
    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<InnerSequence> {
        self.0.borrow_mut()
    }

    pub(crate) fn to_output(&self) -> OutputSequence {
        let s = self.0.borrow();
        OutputSequence {
            items: s.items.iter().map(|i| i.to_output()).collect(),
        }
    }

    pub(crate) fn to_one(&self) -> Result<Item> {
        let s = self.0.borrow();
        if s.len() == 1 {
            Ok(s.items[0].clone())
        } else {
            Err(ValueError::Type)
        }
    }

    pub(crate) fn to_option(&self) -> Result<Option<Item>> {
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
pub(crate) struct InnerSequence {
    pub(crate) items: Vec<Item>,
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

    pub fn as_slice(&self) -> &[Item] {
        &self.items
    }

    pub(crate) fn from_items(items: &[Item]) -> Self {
        Self {
            items: items.to_vec(),
        }
    }

    pub(crate) fn from_vec(items: Vec<Item>) -> Self {
        Self { items }
    }

    pub(crate) fn from_item(item: Item) -> Self {
        Self { items: vec![item] }
    }

    pub(crate) fn from_atomic(atomic: Atomic) -> Self {
        if matches!(atomic, Atomic::Empty) {
            return Self::new();
        }
        Self {
            items: vec![Item::Atomic(atomic)],
        }
    }

    pub(crate) fn from_node(node: Node) -> Self {
        Self {
            items: vec![Item::Node(node)],
        }
    }

    pub(crate) fn singleton(&self) -> Result<&Item> {
        if self.items.len() == 1 {
            Ok(&self.items[0])
        } else {
            Err(ValueError::Type)
        }
    }

    pub(crate) fn push_value(&mut self, value: Value) {
        match value {
            Value::Atomic(a) => self.items.push(Item::Atomic(a)),
            Value::Closure(c) => self.items.push(Item::Function(c)),
            Value::Sequence(s) => self.extend(s),
            Value::Node(n) => self.items.push(Item::Node(n)),
            _ => panic!("unexpected value: {:?}", value),
        }
    }

    pub(crate) fn push(&mut self, item: &Item) {
        self.items.push(item.clone());
    }

    pub(crate) fn extend(&mut self, other: Sequence) {
        for item in &other.borrow().items {
            self.push(item);
        }
    }

    pub(crate) fn atomize(&self, xot: &Xot) -> InnerSequence {
        let mut items = Vec::new();
        for item in &self.items {
            match item {
                Item::Atomic(a) => items.push(Item::Atomic(a.clone())),
                Item::Node(n) => {
                    for typed_value in n.typed_value(xot) {
                        items.push(Item::Atomic(typed_value));
                    }
                }
                // XXX need code to handle array case
                Item::Function(..) => panic!("cannot atomize a function"),
            }
        }
        InnerSequence { items }
    }

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> Result<Atomic> {
        // we avoid using atomize as an optimization, so we don't
        // have to atomize all entries just to get the first item
        match self.len() {
            0 => Ok(Atomic::Empty),
            1 => {
                let item = &self.items[0];
                match item {
                    Item::Atomic(a) => Ok(a.clone()),
                    Item::Node(n) => {
                        let mut t = n.typed_value(context.xot);
                        if t.len() != 1 {
                            return Err(ValueError::XPTY0004);
                        }
                        Ok(t.remove(0))
                    }
                    // XXX need code to handle array case
                    Item::Function(..) => panic!("cannot atomize a function"),
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
                Item::Atomic(a) => atoms.push(a),
                _ => unreachable!("atomize returned a non-atomic item"),
            }
        }
        atoms
    }

    pub(crate) fn concat(&self, other: &InnerSequence) -> InnerSequence {
        let mut items = self.items.clone();
        items.extend(other.items.clone());
        InnerSequence { items }
    }

    pub(crate) fn union(
        &self,
        other: &InnerSequence,
        annotations: &Annotations,
    ) -> Result<InnerSequence> {
        let mut s = HashSet::new();
        for item in &self.items {
            let node = match item {
                Item::Node(node) => *node,
                Item::Atomic(..) => return Err(ValueError::Type),
                Item::Function(..) => return Err(ValueError::Type),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                Item::Node(node) => *node,
                Item::Atomic(..) => return Err(ValueError::Type),
                Item::Function(..) => return Err(ValueError::Type),
            };
            s.insert(node);
        }

        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));

        let items = nodes.into_iter().map(Item::Node).collect::<Vec<_>>();
        Ok(InnerSequence { items })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputSequence {
    items: Vec<OutputItem>,
}

impl OutputSequence {
    pub fn new(items: Vec<OutputItem>) -> Self {
        Self { items }
    }

    pub fn items(&self) -> &[OutputItem] {
        &self.items
    }

    // XXX unfortunate duplication with effective_boolean_value
    // on Value
    pub fn effective_boolean_value(&self) -> std::result::Result<bool, crate::error::Error> {
        if self.items.is_empty() {
            return Ok(false);
        }
        if matches!(self.items[0], OutputItem::Node(_)) {
            return Ok(true);
        }
        if self.items.len() != 1 {
            return Err(crate::Error::FORG0006);
        }
        match self.items[0].to_bool() {
            Ok(b) => Ok(b),
            Err(_) => Err(crate::Error::FORG0006),
        }
    }
}
