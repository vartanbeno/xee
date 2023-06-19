use xot::Xot;

use crate::stack;
use crate::xml;

#[derive(Clone)]
pub(crate) enum AtomizedIter<'a> {
    Atomic(AtomizedAtomicIter),
    Node(AtomizedNodeIter),
    Sequence(AtomizedSequenceIter<'a>),
    Erroring(ErroringAtomizedIter),
}

impl<'a> AtomizedIter<'a> {
    pub(crate) fn new(value: stack::Value, xot: &'a Xot) -> AtomizedIter<'a> {
        match value {
            stack::Value::Atomic(atomic) => AtomizedIter::Atomic(AtomizedAtomicIter::new(atomic)),
            stack::Value::Node(node) => AtomizedIter::Node(AtomizedNodeIter::new(node, xot)),
            stack::Value::Sequence(sequence) => {
                AtomizedIter::Sequence(AtomizedSequenceIter::new(sequence, xot))
            }
            stack::Value::Closure(_) => AtomizedIter::Erroring(ErroringAtomizedIter {}),
            stack::Value::Step(_) => AtomizedIter::Erroring(ErroringAtomizedIter {}),
        }
    }
}

impl Iterator for AtomizedIter<'_> {
    type Item = stack::Result<stack::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AtomizedIter::Atomic(iter) => iter.next().map(Ok),
            AtomizedIter::Node(iter) => iter.next().map(Ok),
            AtomizedIter::Sequence(iter) => iter.next(),
            AtomizedIter::Erroring(iter) => iter.next(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomizedAtomicIter {
    atomic: stack::Atomic,
    done: bool,
}

impl AtomizedAtomicIter {
    fn new(atomic: stack::Atomic) -> Self {
        Self {
            atomic,
            done: false,
        }
    }
}

impl Iterator for AtomizedAtomicIter {
    type Item = stack::Atomic;

    fn next(&mut self) -> Option<Self::Item> {
        if matches!(self.atomic, stack::Atomic::Empty) {
            return None;
        }
        if self.done {
            None
        } else {
            self.done = true;
            Some(self.atomic.clone())
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomizedNodeIter {
    typed_value: Vec<stack::Atomic>,
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
    type Item = stack::Atomic;

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
pub struct AtomizedSequenceIter<'a> {
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
    type Item = stack::Result<stack::Atomic>;

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
                stack::Item::Function(..) => return Some(Err(stack::Error::Type)),
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct ErroringAtomizedIter;

impl Iterator for ErroringAtomizedIter {
    type Item = stack::Result<stack::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Err(stack::Error::Type))
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
        let atomic = stack::Atomic::Integer(3);
        let value = stack::Value::Atomic(atomic.clone());

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
        let value = stack::Value::Node(node);

        let mut iter = AtomizedIter::new(value, &xot);

        assert_eq!(
            iter.next(),
            Some(Ok(stack::Atomic::String(Rc::new("Hello".to_string()))))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_atomize_sequence() {
        let mut xot = Xot::new();
        let root = xot.parse("<doc>Hello</doc>").unwrap();
        let xot_node = xot.document_element(root).unwrap();
        let node = xml::Node::Xot(xot_node);
        let value = stack::Value::Sequence(stack::Sequence::from_items(&[
            stack::Item::Atomic(stack::Atomic::Integer(3)),
            stack::Item::Node(node),
            stack::Item::Atomic(stack::Atomic::Integer(4)),
        ]));

        let mut iter = AtomizedIter::new(value, &xot);

        assert_eq!(iter.next(), Some(Ok(stack::Atomic::Integer(3))));
        assert_eq!(
            iter.next(),
            Some(Ok(stack::Atomic::String(Rc::new("Hello".to_string()))))
        );
        assert_eq!(iter.next(), Some(Ok(stack::Atomic::Integer(4))));
        assert_eq!(iter.next(), None);
    }
}
