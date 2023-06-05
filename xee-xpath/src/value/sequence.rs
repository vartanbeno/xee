use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec;
use xot::Xot;

use crate::annotation::Annotations;
use crate::context::DynamicContext;
use crate::value::atomic::Atomic;
use crate::value::error::ValueError;
use crate::value::item::Item;
use crate::value::node::Node;
use crate::value::value::Value;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub(crate) items: Vec<Item>,
}

impl Sequence {
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

    pub(crate) fn from_vec(items: Vec<Item>) -> Self {
        Self { items }
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

    pub(crate) fn extend(&mut self, other: Rc<RefCell<Sequence>>) {
        for item in &other.borrow().items {
            self.push(item);
        }
    }

    pub(crate) fn atomize(&self, xot: &Xot) -> Sequence {
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
        Sequence { items }
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

    pub(crate) fn concat(&self, other: &Sequence) -> Sequence {
        let mut items = self.items.clone();
        items.extend(other.items.clone());
        Sequence { items }
    }

    pub(crate) fn union(&self, other: &Sequence, annotations: &Annotations) -> Result<Sequence> {
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
        Ok(Sequence { items })
    }
}
