use ahash::{HashSet, HashSetExt};
use std::cell::RefCell;
use std::rc::Rc;

use crate::error;
use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BuildSequence(Rc<RefCell<InnerSequence>>);

impl BuildSequence {
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

    pub(crate) fn into_stack_value(self) -> stack::Value {
        let inner = Rc::into_inner(self.0).unwrap();
        inner.into_inner().into_stack_value()
    }
}

impl From<BuildSequence> for stack::Value {
    fn from(sequence: BuildSequence) -> Self {
        sequence.into_stack_value()
    }
}

// impl TryFrom<stack::Value> for stack::Sequence {
//     type Error = error::Error;

//     fn try_from(value: stack::Value) -> error::Result<Self> {
//         value.to_sequence()
//     }
// }

// impl TryFrom<&stack::Value> for stack::Sequence {
//     type Error = error::Error;

//     fn try_from(value: &stack::Value) -> error::Result<Self> {
//         value.to_sequence()
//     }
// }

// impl From<Vec<stack::Item>> for stack::Sequence {
//     fn from(items: Vec<stack::Item>) -> Self {
//         Self::new(InnerSequence::new(items))
//     }
// }

// impl From<stack::Item> for stack::Sequence {
//     fn from(item: stack::Item) -> Self {
//         Self::new(InnerSequence::new(vec![item]))
//     }
// }

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

    pub(crate) fn push_value(&mut self, value: stack::Value) {
        match value {
            stack::Value::Empty => {}
            stack::Value::Item(item) => self.items.push(item),
            stack::Value::Many(items) => self.items.extend(items.as_ref().into_iter().cloned()),
            stack::Value::Absent => panic!("Don't know how to handle absent"),
            stack::Value::Build(_) => unreachable!(),
        }
    }

    pub(crate) fn push(&mut self, item: &stack::Item) {
        self.items.push(item.clone());
    }

    pub(crate) fn into_stack_value(mut self) -> stack::Value {
        if self.items.is_empty() {
            stack::Value::Empty
        } else if self.items.len() == 1 {
            stack::Value::Item(self.items.pop().unwrap())
        } else {
            stack::Value::Many(Rc::new(self.items))
        }
    }
}
