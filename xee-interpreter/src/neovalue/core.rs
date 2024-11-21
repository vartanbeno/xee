use std::rc::Rc;

use xot::Node;

use crate::{atomic, error, function, sequence::Item};

use super::{
    iter::NodeIter,
    traits::{Sequence, SequenceExt},
};

#[derive(Debug, Clone)]
struct Empty {}

impl<'a> Sequence<'a, std::iter::Empty<&'a Item>> for Empty {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn get(&self, _index: usize) -> Option<&Item> {
        None
    }

    #[inline]
    fn items(&self) -> std::iter::Empty<&'a Item> {
        std::iter::empty()
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        Ok(false)
    }

    #[inline]
    fn string_value(&self, _xot: &xot::Xot) -> error::Result<String> {
        Ok(String::new())
    }
}

#[derive(Debug, Clone)]
struct One {
    item: Item,
}

impl<'a> Sequence<'a, std::iter::Once<&'a Item>> for One {
    #[inline]
    fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    fn len(&self) -> usize {
        1
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&Item> {
        if index == 0 {
            Some(&self.item)
        } else {
            None
        }
    }

    #[inline]
    fn items(&'a self) -> std::iter::Once<&'a Item> {
        std::iter::once(&self.item)
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        self.item.effective_boolean_value()
    }

    #[inline]
    fn string_value(&self, xot: &xot::Xot) -> error::Result<String> {
        self.item.string_value(xot)
    }
}

#[derive(Debug, Clone)]
struct Many {
    items: Rc<Vec<Item>>,
}

impl<'a> Sequence<'a, std::slice::Iter<'a, Item>> for Many {
    #[inline]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    #[inline]
    fn items(&'a self) -> std::slice::Iter<'a, Item> {
        self.items.iter()
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn string_value(&self, _xot: &xot::Xot) -> error::Result<String> {
        Err(error::Error::XPTY0004)
    }
}

// specifically implement the extensions for each version, so that
// we can avoid dynamic dispatch on the inside
impl<'a, I> SequenceExt<'a, I> for Empty
where
    I: Iterator<Item = &'a Item>,
    Empty: Sequence<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for One
where
    I: Iterator<Item = &'a Item>,
    One: Sequence<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for Many
where
    I: Iterator<Item = &'a Item>,
    Many: Sequence<'a, I>,
{
}

#[derive(Debug, Clone)]
pub enum StackSequence {
    Empty(Empty),
    One(One),
    Many(Many),
}

impl<'a> Sequence<'a, Box<dyn Iterator<Item = &'a Item> + 'a>> for StackSequence {
    fn is_empty(&self) -> bool {
        match self {
            StackSequence::Empty(inner) => inner.is_empty(),
            StackSequence::One(inner) => inner.is_empty(),
            StackSequence::Many(inner) => inner.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            StackSequence::Empty(inner) => inner.len(),
            StackSequence::One(inner) => inner.len(),
            StackSequence::Many(inner) => inner.len(),
        }
    }

    fn get(&self, index: usize) -> Option<&Item> {
        match self {
            StackSequence::Empty(inner) => inner.get(index),
            StackSequence::One(inner) => inner.get(index),
            StackSequence::Many(inner) => inner.get(index),
        }
    }

    fn items(&'a self) -> Box<dyn Iterator<Item = &'a Item> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.items()),
            StackSequence::One(inner) => Box::new(inner.items()),
            StackSequence::Many(inner) => Box::new(inner.items()),
        }
    }

    fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            StackSequence::Empty(inner) => inner.effective_boolean_value(),
            StackSequence::One(inner) => inner.effective_boolean_value(),
            StackSequence::Many(inner) => inner.effective_boolean_value(),
        }
    }

    fn string_value(&self, xot: &xot::Xot) -> error::Result<String> {
        match self {
            StackSequence::Empty(inner) => inner.string_value(xot),
            StackSequence::One(inner) => inner.string_value(xot),
            StackSequence::Many(inner) => inner.string_value(xot),
        }
    }
}

// we implement these explicitly, because we want to avoid dynamic dispatch until
// the outer layer. This gives the compiler the chance to optimize the inner
// layers better.
impl<'a> SequenceExt<'a, Box<dyn Iterator<Item = &'a Item>>> for StackSequence
where
    StackSequence: Sequence<'a, Box<dyn Iterator<Item = &'a Item>>>,
{
    #[allow(refining_impl_trait)]
    fn nodes(&'a self) -> Box<dyn Iterator<Item = error::Result<xot::Node>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.nodes()),
            StackSequence::One(inner) => Box::new(inner.nodes()),
            StackSequence::Many(inner) => Box::new(inner.nodes()),
        }
    }

    #[allow(refining_impl_trait)]
    fn atomized(
        &'a self,
        xot: &'a xot::Xot,
    ) -> Box<dyn Iterator<Item = error::Result<atomic::Atomic>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.atomized(xot)),
            StackSequence::One(inner) => Box::new(inner.atomized(xot)),
            StackSequence::Many(inner) => Box::new(inner.atomized(xot)),
        }
    }

    #[allow(refining_impl_trait)]
    fn map_iter(&'a self) -> Box<dyn Iterator<Item = error::Result<function::Map>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.map_iter()),
            StackSequence::One(inner) => Box::new(inner.map_iter()),
            StackSequence::Many(inner) => Box::new(inner.map_iter()),
        }
    }

    #[allow(refining_impl_trait)]
    fn array_iter(&'a self) -> Box<dyn Iterator<Item = error::Result<function::Array>> + 'a> {
        match self {
            StackSequence::Empty(inner) => Box::new(inner.array_iter()),
            StackSequence::One(inner) => Box::new(inner.array_iter()),
            StackSequence::Many(inner) => Box::new(inner.array_iter()),
        }
    }

    #[allow(refining_impl_trait)]
    fn elements(
        &'a self,
        xot: &'a xot::Xot,
    ) -> error::Result<Box<dyn Iterator<Item = error::Result<xot::Node>> + 'a>> {
        match self {
            StackSequence::Empty(inner) => Ok(Box::new(inner.elements(xot)?)),
            StackSequence::One(inner) => Ok(Box::new(inner.elements(xot)?)),
            StackSequence::Many(inner) => Ok(Box::new(inner.elements(xot)?)),
        }
    }

    #[allow(refining_impl_trait)]
    fn to_array(&'a self) -> error::Result<function::Array> {
        match self {
            StackSequence::Empty(inner) => inner.to_array(),
            StackSequence::One(inner) => inner.to_array(),
            StackSequence::Many(inner) => inner.to_array(),
        }
    }
}
