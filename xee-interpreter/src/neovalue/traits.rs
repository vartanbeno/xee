use xot::Xot;

use crate::{
    atomic::{self, AtomicCompare},
    context, error, function,
    sequence::Item,
    xml,
};

use super::{
    comparison,
    iter::{AtomizedIter, NodeIter},
};

/// The core sequence interface: a sequence must implement this to function.
///
/// If you do, SequenceExt provides a whole of APIs on top of it.
pub trait Sequence<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    /// Check whether the sequence is empty
    fn is_empty(&self) -> bool;

    /// Get an item in the sequenc
    fn len(&self) -> usize;

    /// Get an item in the index, if it exists
    fn get(&'a self, index: usize) -> Option<&'a Item>;

    /// Get the items from the sequence as an iterator
    fn items(&'a self) -> I;

    /// Effective boolean value
    fn effective_boolean_value(&'a self) -> error::Result<bool>;

    /// String value
    fn string_value(&'a self, xot: &Xot) -> error::Result<String>;
}

pub trait SequenceExt<'a, I>: Sequence<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    /// Access an iterator over the nodes in the sequence
    ///
    /// An error is returned for items that are not a node.
    fn nodes(&'a self) -> impl Iterator<Item = error::Result<xot::Node>> {
        NodeIter::new(self.items())
    }

    /// Access an iterator for the atomized values in the sequence
    fn atomized(&'a self, xot: &'a Xot) -> impl Iterator<Item = error::Result<atomic::Atomic>> {
        AtomizedIter::new(xot, self.items())
    }

    /// Access an iterator over the XPath maps in the sequence
    ///
    /// An error is returned for items that are not a map.
    fn map_iter(&'a self) -> impl Iterator<Item = error::Result<function::Map>> {
        self.items().map(|item| item.to_map())
    }

    /// Access an iterator over the XPath arrays in the sequence
    ///
    /// An error is returned for items that are not an array.
    fn array_iter(&'a self) -> impl Iterator<Item = error::Result<function::Array>> {
        self.items().map(|item| item.to_array())
    }

    /// Access an iterator over elements nodes in the sequence
    ///
    /// An error is returned for items that are not an element.
    fn elements(
        &'a self,
        xot: &'a Xot,
    ) -> error::Result<impl Iterator<Item = error::Result<xot::Node>>> {
        Ok(self.nodes().map(|n| match n {
            Ok(n) => {
                if xot.is_element(n) {
                    Ok(n)
                } else {
                    Err(error::Error::XPTY0004)
                }
            }
            Err(n) => Err(n),
        }))
    }

    /// Create an XPath array from this sequence.
    fn to_array(&'a self) -> error::Result<function::Array> {
        let mut array = Vec::with_capacity(self.len());
        for item in self.items() {
            array.push(item.clone().into());
        }
        // TODO: array.into() is somehow returning a Result, that seems weird
        // if this is really fallible, it should be try into. If it's not,
        // this whole function should be infallible.
        Ok(array.into())
    }
}

pub(crate) trait SequenceCompare<'a, I>: SequenceExt<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    fn general_comparison<O>(
        &'a self,
        other: &'a impl SequenceExt<'a, I>,
        context: &context::DynamicContext,
        xot: &'a Xot,
        op: O,
    ) -> error::Result<bool>
    where
        O: AtomicCompare,
    {
        comparison::general_comparison(self.atomized(xot), other.atomized(xot), context, op)
    }
}

pub(crate) trait SequenceOrder<'a, I>: Sequence<'a, I>
where
    I: Iterator<Item = &'a Item>,
{
    fn one_node(&'a self) -> error::Result<xot::Node> {
        match self.len() {
            1 => self.items().next().unwrap().to_node(),
            _ => Err(error::Error::XPTY0004),
        }
    }

    fn is(
        &'a self,
        other: &'a impl SequenceOrder<'a, I>,
        annotations: &xml::Annotations,
    ) -> error::Result<bool> {
        let a = self.one_node()?;
        let b = other.one_node()?;
        let a_annotation = annotations.get(a).unwrap();
        let b_annotation = annotations.get(b).unwrap();
        Ok(a_annotation.document_order == b_annotation.document_order)
    }

    fn precedes(
        &'a self,
        other: &'a impl SequenceOrder<'a, I>,
        annotations: &xml::Annotations,
    ) -> error::Result<bool> {
        let a = self.one_node()?;
        let b = other.one_node()?;
        let a_annotation = annotations.get(a).unwrap();
        let b_annotation = annotations.get(b).unwrap();
        Ok(a_annotation.document_order < b_annotation.document_order)
    }

    fn follows(
        &'a self,
        other: &'a impl SequenceOrder<'a, I>,
        annotations: &xml::Annotations,
    ) -> error::Result<bool> {
        let a = self.one_node()?;
        let b = other.one_node()?;
        let a_annotation = annotations.get(a).unwrap();
        let b_annotation = annotations.get(b).unwrap();
        Ok(a_annotation.document_order > b_annotation.document_order)
    }
}

// impl<'a, T, I> SequenceExt<'a, I> for T
// where
//     T: Sequence<'a, I>,
//     I: Iterator<Item = &'a Item>,
// {
// }

// impl<'a, T, I> SequenceCompare<'a, I> for T
// where
//     T: Sequence<'a, I>,
//     I: Iterator<Item = &'a Item>,
// {
// }

// impl<'a, T, I> SequenceOrder<'a, I> for T
// where
//     T: Sequence<'a, I>,
//     I: Iterator<Item = &'a Item>,
// {
// }
