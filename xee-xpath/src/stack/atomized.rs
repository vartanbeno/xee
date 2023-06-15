use xot::Xot;

use crate::stack;
use crate::xml;

struct Atomized<'a> {
    xot: &'a Xot,
    value: stack::Value,
}

impl<'a> Atomized<'a> {
    fn new(value: stack::Value, xot: &'a Xot) -> Self {
        Self { value, xot }
    }

    fn into_iter(self) -> AtomizedIter<'a> {
        match self.value {
            stack::Value::Atomic(atomic) => AtomizedIter::Atomic(AtomizedAtomicIter::new(atomic)),
            stack::Value::Node(node) => AtomizedIter::Node(AtomizedNodeIter::new(node, self.xot)),
            stack::Value::Sequence(sequence) => {
                AtomizedIter::Sequence(AtomizedSequenceIter::new(sequence, self.xot))
            }
            stack::Value::Closure(_) => {
                // TODO array case?
                panic!("need to handle atomizing a function");
            }
            stack::Value::Step(_) => {
                panic!("cannot atomized step");
            }
        }
    }
}

enum AtomizedIter<'a> {
    Atomic(AtomizedAtomicIter),
    Node(AtomizedNodeIter),
    Sequence(AtomizedSequenceIter<'a>),
}

impl Iterator for AtomizedIter<'_> {
    type Item = stack::Atomic;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AtomizedIter::Atomic(iter) => iter.next(),
            AtomizedIter::Node(iter) => iter.next(),
            AtomizedIter::Sequence(iter) => iter.next(),
        }
    }
}

struct AtomizedAtomicIter {
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

struct AtomizedNodeIter {
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

struct AtomizedSequenceIter<'a> {
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
    type Item = stack::Atomic;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.sequence.len() {
            // if there are any more atomized nodes to iterate, do that
            if let Some(node_iter) = &mut self.node_iter {
                if let Some(item) = node_iter.next() {
                    return Some(item);
                } else {
                    self.index += 1;
                    self.node_iter = None;
                }
            }

            let item = &self.sequence.borrow().items[self.index];
            match item {
                stack::Item::Atomic(a) => {
                    self.index += 1;
                    return Some(a.clone());
                }
                stack::Item::Node(n) => {
                    self.node_iter = Some(AtomizedNodeIter::new(*n, self.xot));
                    continue;
                }
                // TODO: needs to handle the array case
                stack::Item::Function(..) => panic!("cannot atomize a function yet"),
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
        let atomic = stack::Atomic::Integer(3);
        let value = stack::Value::Atomic(atomic.clone());

        let atomized = Atomized::new(value, &xot);
        let mut iter = atomized.into_iter();
        assert_eq!(iter.next(), Some(atomic));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_atomize_node() {
        let mut xot = Xot::new();
        let root = xot.parse("<doc>Hello</doc>").unwrap();
        let xot_node = xot.document_element(root).unwrap();
        let node = xml::Node::Xot(xot_node);
        let value = stack::Value::Node(node);

        let atomized = Atomized::new(value, &xot);
        let mut iter = atomized.into_iter();

        assert_eq!(
            iter.next(),
            Some(stack::Atomic::String(Rc::new("Hello".to_string())))
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

        let atomized = Atomized::new(value, &xot);
        let mut iter = atomized.into_iter();

        assert_eq!(iter.next(), Some(stack::Atomic::Integer(3)));
        assert_eq!(
            iter.next(),
            Some(stack::Atomic::String(Rc::new("Hello".to_string())))
        );
        assert_eq!(iter.next(), Some(stack::Atomic::Integer(4)));
        assert_eq!(iter.next(), None);
    }
}
