use xot::Xot;

use crate::atomic;
use crate::error;
use crate::stack;
use crate::xml;

#[derive(Clone)]
pub(crate) enum AtomizedIter<'a> {
    Empty,
    // TODO: introduce AtomizedItemIter
    Atomic(std::iter::Once<error::Result<atomic::Atomic>>),
    Node(AtomizedNodeIter),
    Sequence(AtomizedSequenceIter<'a>),
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
    Absent(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl<'a> AtomizedIter<'a> {
    pub(crate) fn new(value: stack::Value, xot: &'a Xot) -> AtomizedIter<'a> {
        match value {
            stack::Value::Empty => AtomizedIter::Empty,
            stack::Value::Item(item) => match item {
                stack::Item::Atomic(atomic) => AtomizedIter::Atomic(std::iter::once(Ok(atomic))),
                stack::Item::Node(node) => AtomizedIter::Node(AtomizedNodeIter::new(node, xot)),
                stack::Item::Function(_) => {
                    AtomizedIter::Erroring(std::iter::once(Err(error::Error::Type)))
                }
            },
            stack::Value::Sequence(sequence) => {
                AtomizedIter::Sequence(AtomizedSequenceIter::new(sequence, xot))
            }
            stack::Value::Absent => AtomizedIter::Absent(std::iter::once(Err(
                error::Error::ComponentAbsentInDynamicContext,
            ))),
        }
    }
}

impl Iterator for AtomizedIter<'_> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AtomizedIter::Empty => None,
            AtomizedIter::Atomic(iter) => iter.next(),
            AtomizedIter::Node(iter) => iter.next().map(Ok),
            AtomizedIter::Sequence(iter) => iter.next(),
            AtomizedIter::Erroring(iter) => iter.next(),
            AtomizedIter::Absent(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AtomizedNodeIter {
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
pub(crate) struct AtomizedSequenceIter<'a> {
    xot: &'a Xot,
    sequence: stack::Sequence,
    index: usize,
    node_iter: Option<AtomizedNodeIter>,
}

impl<'a> AtomizedSequenceIter<'a> {
    fn new(sequence: stack::Sequence, xot: &'a Xot) -> Self {
        Self {
            xot,
            sequence,
            index: 0,
            node_iter: None,
        }
    }
}

impl<'a> Iterator for AtomizedSequenceIter<'a> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.sequence.len() {
            // if there are any more atomized nodes to iterate, do that
            if let Some(node_iter) = &mut self.node_iter {
                if let Some(item) = node_iter.next() {
                    return Some(Ok(item));
                } else {
                    self.index += 1;
                    self.node_iter = None;
                    continue;
                }
            }

            let item = &self.sequence.borrow().items[self.index];
            match item {
                stack::Item::Atomic(a) => {
                    self.index += 1;
                    return Some(Ok(a.clone()));
                }
                stack::Item::Node(n) => {
                    self.node_iter = Some(AtomizedNodeIter::new(*n, self.xot));
                    continue;
                }
                // TODO: needs to handle the array case
                stack::Item::Function(..) => return Some(Err(error::Error::Type)),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    use crate::stack;

    #[test]
    fn test_atomize_atomic() {
        let xot = Xot::new();
        let atomic = atomic::Atomic::Integer(3);
        let value = 3i64.into();

        let mut iter = AtomizedIter::new(value, &xot);
        assert_eq!(iter.next(), Some(Ok(atomic)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_atomize_node() {
        let mut xot = Xot::new();
        let root = xot.parse("<doc>Hello</doc>").unwrap();
        let xot_node = xot.document_element(root).unwrap();
        let node = xml::Node::Xot(xot_node);
        let value = node.into();

        let mut iter = AtomizedIter::new(value, &xot);

        assert_eq!(
            iter.next(),
            Some(Ok(atomic::Atomic::String(Rc::new("Hello".to_string()))))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_atomize_sequence() {
        let mut xot = Xot::new();
        let root = xot.parse("<doc>Hello</doc>").unwrap();
        let xot_node = xot.document_element(root).unwrap();
        let node = xml::Node::Xot(xot_node);
        let value = stack::Value::Sequence(stack::Sequence::from(vec![
            stack::Item::Atomic(atomic::Atomic::Integer(3)),
            stack::Item::Node(node),
            stack::Item::Atomic(atomic::Atomic::Integer(4)),
        ]));

        let mut iter = AtomizedIter::new(value, &xot);

        assert_eq!(iter.next(), Some(Ok(atomic::Atomic::Integer(3))));
        assert_eq!(
            iter.next(),
            Some(Ok(atomic::Atomic::String(Rc::new("Hello".to_string()))))
        );
        assert_eq!(iter.next(), Some(Ok(atomic::Atomic::Integer(4))));
        assert_eq!(iter.next(), None);
    }
}
