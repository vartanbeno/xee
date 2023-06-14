use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec;
use xot::Xot;

use crate::annotation::Annotations;
use crate::context::DynamicContext;
use crate::data::Node;
use crate::data::OutputSequence;
use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StackSequence(Rc<RefCell<StackInnerSequence>>);

impl StackSequence {
    pub(crate) fn new(sequence: StackInnerSequence) -> Self {
        Self(Rc::new(RefCell::new(sequence)))
    }
    pub(crate) fn empty() -> Self {
        Self::new(StackInnerSequence::new())
    }
    pub(crate) fn from_atomic(atomic: &stack::Atomic) -> Self {
        Self::new(StackInnerSequence::from_atomic(atomic.clone()))
    }
    pub(crate) fn from_node(node: Node) -> Self {
        Self::new(StackInnerSequence::from_node(node))
    }
    pub(crate) fn from_vec(items: Vec<stack::StackItem>) -> Self {
        Self::new(StackInnerSequence::from_vec(items))
    }
    pub(crate) fn from_items(items: &[stack::StackItem]) -> Self {
        Self::new(StackInnerSequence::from_items(items))
    }

    pub(crate) fn from_item(item: stack::StackItem) -> Self {
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

    pub(crate) fn to_one(&self) -> stack::ValueResult<stack::StackItem> {
        let s = self.0.borrow();
        if s.len() == 1 {
            Ok(s.items[0].clone())
        } else {
            Err(stack::ValueError::Type)
        }
    }

    pub(crate) fn to_option(&self) -> stack::ValueResult<Option<stack::StackItem>> {
        let s = self.0.borrow();
        if s.len() == 1 {
            Ok(Some(s.items[0].clone()))
        } else if s.is_empty() {
            Ok(None)
        } else {
            Err(stack::ValueError::Type)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StackInnerSequence {
    pub(crate) items: Vec<stack::StackItem>,
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

    pub fn as_slice(&self) -> &[stack::StackItem] {
        &self.items
    }

    pub(crate) fn from_items(items: &[stack::StackItem]) -> Self {
        Self {
            items: items.to_vec(),
        }
    }

    pub(crate) fn from_vec(items: Vec<stack::StackItem>) -> Self {
        Self { items }
    }

    pub(crate) fn from_item(item: stack::StackItem) -> Self {
        Self { items: vec![item] }
    }

    pub(crate) fn from_atomic(atomic: stack::Atomic) -> Self {
        if matches!(atomic, stack::Atomic::Empty) {
            return Self::new();
        }
        Self {
            items: vec![stack::StackItem::Atomic(atomic)],
        }
    }

    pub(crate) fn from_node(node: Node) -> Self {
        Self {
            items: vec![stack::StackItem::Node(node)],
        }
    }

    pub(crate) fn singleton(&self) -> stack::ValueResult<&stack::StackItem> {
        if self.items.len() == 1 {
            Ok(&self.items[0])
        } else {
            Err(stack::ValueError::Type)
        }
    }

    pub(crate) fn push_value(&mut self, value: stack::StackValue) {
        match value {
            stack::StackValue::Atomic(a) => self.items.push(stack::StackItem::Atomic(a)),
            stack::StackValue::Closure(c) => self.items.push(stack::StackItem::Function(c)),
            stack::StackValue::Sequence(s) => self.extend(s),
            stack::StackValue::Node(n) => self.items.push(stack::StackItem::Node(n)),
            _ => panic!("unexpected value: {:?}", value),
        }
    }

    pub(crate) fn push(&mut self, item: &stack::StackItem) {
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
                stack::StackItem::Atomic(a) => items.push(stack::StackItem::Atomic(a.clone())),
                stack::StackItem::Node(n) => {
                    for typed_value in n.typed_value(xot) {
                        items.push(stack::StackItem::Atomic(typed_value));
                    }
                }
                // XXX need code to handle array case
                stack::StackItem::Function(..) => panic!("cannot atomize a function"),
            }
        }
        StackInnerSequence { items }
    }

    pub(crate) fn to_atomic(&self, context: &DynamicContext) -> stack::ValueResult<stack::Atomic> {
        // we avoid using atomize as an optimization, so we don't
        // have to atomize all entries just to get the first item
        match self.len() {
            0 => Ok(stack::Atomic::Empty),
            1 => {
                let item = &self.items[0];
                match item {
                    stack::StackItem::Atomic(a) => Ok(a.clone()),
                    stack::StackItem::Node(n) => {
                        let mut t = n.typed_value(context.xot);
                        if t.len() != 1 {
                            return Err(stack::ValueError::XPTY0004);
                        }
                        Ok(t.remove(0))
                    }
                    // XXX need code to handle array case
                    stack::StackItem::Function(..) => panic!("cannot atomize a function"),
                }
            }
            _ => Err(stack::ValueError::XPTY0004),
        }
    }

    pub(crate) fn to_atoms(&self, xot: &Xot) -> Vec<stack::Atomic> {
        let mut atoms = Vec::new();
        let atomized = self.atomize(xot);
        for item in atomized.items {
            match item {
                stack::StackItem::Atomic(a) => atoms.push(a),
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
    ) -> stack::ValueResult<StackInnerSequence> {
        let mut s = HashSet::new();
        for item in &self.items {
            let node = match item {
                stack::StackItem::Node(node) => *node,
                stack::StackItem::Atomic(..) => return Err(stack::ValueError::Type),
                stack::StackItem::Function(..) => return Err(stack::ValueError::Type),
            };
            s.insert(node);
        }
        for item in &other.items {
            let node = match item {
                stack::StackItem::Node(node) => *node,
                stack::StackItem::Atomic(..) => return Err(stack::ValueError::Type),
                stack::StackItem::Function(..) => return Err(stack::ValueError::Type),
            };
            s.insert(node);
        }

        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));

        let items = nodes
            .into_iter()
            .map(stack::StackItem::Node)
            .collect::<Vec<_>>();
        Ok(StackInnerSequence { items })
    }
}
