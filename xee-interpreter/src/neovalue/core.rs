use std::rc::Rc;

use crate::sequence::Item;

use super::traits::Sequence;

#[derive(Debug, Clone)]
struct Empty {}

impl<'a> Sequence<'a, std::iter::Empty<&'a Item>> for Empty {
    fn is_empty(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        0
    }

    fn get(&self, _index: usize) -> Option<&Item> {
        None
    }

    fn items(&self) -> std::iter::Empty<&'a Item> {
        std::iter::empty()
    }
}

#[derive(Debug, Clone)]
struct One {
    item: Item,
}

impl<'a> Sequence<'a, std::iter::Once<&'a Item>> for One {
    fn is_empty(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        1
    }

    fn get(&self, index: usize) -> Option<&Item> {
        if index == 0 {
            Some(&self.item)
        } else {
            None
        }
    }

    fn items(&'a self) -> std::iter::Once<&'a Item> {
        std::iter::once(&self.item)
    }
}

#[derive(Debug, Clone)]
struct Many {
    items: Rc<Vec<Item>>,
}

impl<'a> Sequence<'a, std::slice::Iter<'a, Item>> for Many {
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    fn items(&'a self) -> std::slice::Iter<'a, Item> {
        self.items.iter()
    }
}

#[derive(Debug, Clone)]
pub enum StackSequence {
    Empty(Empty),
    One(One),
    Many(Many),
}

// impl Sequence for StackSequence {
//     fn is_empty(&self) -> bool {
//         match self {
//             StackSequence::Empty(inner) => inner.is_empty(),
//             StackSequence::One(inner) => inner.is_empty(),
//             StackSequence::Many(inner) => inner.is_empty(),
//         }
//     }

//     fn len(&self) -> usize {
//         match self {
//             StackSequence::Empty(inner) => inner.len(),
//             StackSequence::One(inner) => inner.len(),
//             StackSequence::Many(inner) => inner.len(),
//         }
//     }

//     fn get(&self, index: usize) -> Option<&Item> {
//         match self {
//             StackSequence::Empty(inner) => inner.get(index),
//             StackSequence::One(inner) => inner.get(index),
//             StackSequence::Many(inner) => inner.get(index),
//         }
//     }

//     fn items(&self) -> impl Iterator<Item = &Item> {
//         match self {
//             StackSequence::Empty(inner) => inner.items(),
//             StackSequence::One(inner) => inner.items(),
//             StackSequence::Many(inner) => inner.items(),
//         }
//     }
// }
