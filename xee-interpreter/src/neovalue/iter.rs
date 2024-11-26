use xot::Xot;

use crate::{atomic, error, function, sequence::Item};

/// An iterator over the nodes in a sequence.
pub struct NodeIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    iter: I,
}

impl<'a, I> NodeIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'a, I> Iterator for NodeIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    type Item = error::Result<xot::Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();
        next.map(|v| v.to_node())
    }
}

/// An iterator atomizing a sequence.
pub struct AtomizedIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    xot: &'a Xot,
    iter: I,
    item_iter: Option<AtomizedItemIter<'a, I>>,
}

impl<'a, I> AtomizedIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    pub(crate) fn new(xot: &'a Xot, iter: I) -> AtomizedIter<'a, I> {
        AtomizedIter {
            xot,
            iter,
            item_iter: None,
        }
    }
}

impl<'a, I> Iterator for AtomizedIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<error::Result<atomic::Atomic>> {
        loop {
            // if there there are any more atoms in this node,
            // supply those
            if let Some(item_iter) = &mut self.item_iter {
                if let Some(item) = item_iter.next() {
                    return Some(item);
                } else {
                    self.item_iter = None;
                }
            }
            // if not, move on to the next item
            let item = self.iter.next();
            if let Some(item) = item {
                self.item_iter = Some(AtomizedItemIter::new(item, self.xot));
                continue;
            } else {
                // no more items, we're done
                return None;
            }
        }
    }
}

/// Atomizing an individual item in a sequence.
pub enum AtomizedItemIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    Atomic(std::iter::Once<atomic::Atomic>),
    Node(AtomizedNodeIter),
    Array(AtomizedArrayIter<'a, I>),
    // TODO: properly handle functions; for now they error
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl<'a, I> AtomizedItemIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    pub(crate) fn new(item: &'a Item, xot: &'a Xot) -> Self {
        match item {
            Item::Atomic(a) => Self::Atomic(std::iter::once(a.clone())),
            Item::Node(n) => Self::Node(AtomizedNodeIter::new(*n, xot)),
            Item::Function(function) => match function.as_ref() {
                function::Function::Array(a) => Self::Array(AtomizedArrayIter::new(a.clone(), xot)),
                _ => Self::Erroring(std::iter::once(Err(error::Error::FOTY0013))),
            },
        }
    }
}

impl<'a, I> Iterator for AtomizedItemIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atomic(iter) => iter.next().map(Ok),
            Self::Node(iter) => iter.next().map(Ok),
            Self::Array(iter) => iter.next(),
            Self::Erroring(iter) => iter.next(),
        }
    }
}

/// Atomizing a node
pub struct AtomizedNodeIter {
    typed_value: Vec<atomic::Atomic>,
    typed_value_index: usize,
}

impl AtomizedNodeIter {
    fn new(node: xot::Node, xot: &Xot) -> Self {
        let s = xot.string_value(node);
        let typed_value = vec![atomic::Atomic::Untyped(s.into())];
        Self {
            typed_value,
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

/// Atomizing a XPath array
pub struct AtomizedArrayIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    xot: &'a Xot,
    array: function::Array,
    array_index: usize,
    iter: Option<Box<AtomizedIter<'a, I>>>,
}

impl<'a, I> AtomizedArrayIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    fn new(array: function::Array, xot: &'a Xot) -> Self {
        Self {
            xot,
            array,
            array_index: 0,
            iter: None,
        }
    }
}

impl<'a, I> Iterator for AtomizedArrayIter<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if there there are any more atoms in this array entry,
            // supply those
            if let Some(iter) = &mut self.iter {
                if let Some(item) = iter.next() {
                    return Some(item);
                } else {
                    self.iter = None;
                }
            }
            let array = &self.array.0;
            // if we're at the end of the array, we're done
            if self.array_index >= array.len() {
                return None;
            }
            let sequence = array[self.array_index].clone();
            self.array_index += 1;

            // TODO: we can't wire this up until the new sequence type is
            // in place
            todo!()
            // self.iter = Some(Box::new(sequence.atomized(self.xot)));
        }
    }
}

pub(crate) fn one<'a>(mut iter: impl Iterator<Item = &'a Item>) -> error::Result<&'a Item> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(one)
        } else {
            Err(error::Error::XPTY0004)
        }
    } else {
        Err(error::Error::XPTY0004)
    }
}

pub(crate) fn option<'a>(
    mut iter: impl Iterator<Item = &'a Item>,
) -> error::Result<Option<&'a Item>> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(Some(one))
        } else {
            Err(error::Error::XPTY0004)
        }
    } else {
        Ok(None)
    }
}
