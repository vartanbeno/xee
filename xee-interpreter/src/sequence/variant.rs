use std::rc::Rc;

use crate::error;

use super::item::Item;
use super::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};

#[derive(Debug, Clone, PartialEq)]
pub struct Empty {}

impl<'a> SequenceCore<'a, std::iter::Empty<&'a Item>> for Empty {
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
    fn one(&self) -> error::Result<&Item> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn option(&self) -> error::Result<Option<&Item>> {
        Ok(None)
    }

    #[inline]
    fn iter(&self) -> std::iter::Empty<&'a Item> {
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

impl IntoIterator for Empty {
    type Item = Item;
    type IntoIter = std::iter::Empty<Item>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct One {
    item: Item,
}

impl One {
    pub(crate) fn item(&self) -> &Item {
        &self.item
    }

    pub(crate) fn into_item(self) -> Item {
        self.item
    }
}

impl From<Item> for One {
    fn from(item: Item) -> Self {
        One { item }
    }
}

impl From<One> for Item {
    fn from(one: One) -> Self {
        one.item
    }
}

impl IntoIterator for One {
    type Item = Item;
    type IntoIter = std::iter::Once<Item>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.item)
    }
}

impl<'a> SequenceCore<'a, std::iter::Once<&'a Item>> for One {
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
    fn one(&self) -> error::Result<&Item> {
        Ok(&self.item)
    }

    #[inline]
    fn option(&self) -> error::Result<Option<&Item>> {
        Ok(Some(&self.item))
    }

    #[inline]
    fn iter(&'a self) -> std::iter::Once<&'a Item> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Many {
    items: Rc<Vec<Item>>,
}

impl Many {}

impl From<Vec<Item>> for Many {
    fn from(items: Vec<Item>) -> Self {
        Many {
            items: Rc::new(items),
        }
    }
}

impl IntoIterator for Many {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Item>;

    // TODO: not the most efficient way to do this, but we use
    // an Rc so we can't just move the items out of the Rc.
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter().cloned().collect::<Vec<_>>().into_iter()
    }
}

impl<'a> SequenceCore<'a, std::slice::Iter<'a, Item>> for Many {
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
    fn one(&self) -> error::Result<&Item> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn option(&self) -> error::Result<Option<&Item>> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn iter(&'a self) -> std::slice::Iter<'a, Item> {
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
// we can avoid dynamic dispatch on the inside. We can't do it generically
// as we want a specialized version for the StackSequence so we can avoid
// dynamic dispatch on the inside.
impl<'a, I> SequenceExt<'a, I> for Empty
where
    I: Iterator<Item = &'a Item> + 'a,
    Empty: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceCompare<'a, I> for Empty
where
    I: Iterator<Item = &'a Item> + 'a,
    Empty: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for Empty
where
    I: Iterator<Item = &'a Item>,
    Empty: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for One
where
    I: Iterator<Item = &'a Item> + 'a,
    One: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceCompare<'a, I> for One
where
    I: Iterator<Item = &'a Item> + 'a,
    One: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for One
where
    I: Iterator<Item = &'a Item>,
    One: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for Many
where
    I: Iterator<Item = &'a Item> + 'a,
    Many: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceCompare<'a, I> for Many
where
    I: Iterator<Item = &'a Item> + 'a,
    Many: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for Many
where
    I: Iterator<Item = &'a Item>,
    Many: SequenceCore<'a, I>,
{
}
