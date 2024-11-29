use std::rc::Rc;

use ibig::IBig;

use crate::{atomic, error};

use super::item::Item;
use super::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};
use super::AtomizedItemIter;

#[derive(Debug, Clone, PartialEq)]
pub struct Empty {}

impl<'a> SequenceCore<'a, std::iter::Empty<Item>> for Empty {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn get(&self, _index: usize) -> Option<Item> {
        None
    }

    #[inline]
    fn one(self) -> error::Result<Item> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item>> {
        Ok(None)
    }

    #[inline]
    fn iter(&self) -> std::iter::Empty<Item> {
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

impl<'a> SequenceCore<'a, std::iter::Once<Item>> for One {
    #[inline]
    fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    fn len(&self) -> usize {
        1
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item> {
        if index == 0 {
            Some(self.item.clone())
        } else {
            None
        }
    }

    #[inline]
    fn one(self) -> error::Result<Item> {
        Ok(self.item)
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item>> {
        Ok(Some(self.item))
    }

    #[inline]
    fn iter(&'a self) -> std::iter::Once<Item> {
        std::iter::once(self.item.clone())
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
    items: Rc<[Item]>,
}

impl Many {}

impl From<Vec<Item>> for Many {
    fn from(items: Vec<Item>) -> Self {
        Many {
            items: items.into(),
        }
    }
}

impl<'a> SequenceCore<'a, std::iter::Cloned<std::slice::Iter<'a, Item>>> for Many {
    #[inline]
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item> {
        self.items.get(index).cloned()
    }

    #[inline]
    fn one(self) -> error::Result<Item> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item>> {
        Err(error::Error::XPTY0004)
    }

    #[inline]
    fn iter(&'a self) -> std::iter::Cloned<std::slice::Iter<'a, Item>> {
        self.items.iter().cloned()
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        // handle the case where the first item is a node
        // it has to be a singleton otherwise
        if matches!(self.items[0], Item::Node(_)) {
            Ok(true)
        } else {
            Err(error::Error::FORG0006)
        }
    }

    #[inline]
    fn string_value(&self, _xot: &xot::Xot) -> error::Result<String> {
        Err(error::Error::XPTY0004)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    start: usize,
    end: usize,
}

impl Range {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Range { start, end }
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }
    pub(crate) fn end(&self) -> usize {
        self.end
    }
}

impl<'a> SequenceCore<'a, RangeIterator> for Range {
    #[inline]
    fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item> {
        if index < self.len() {
            let i: IBig = (self.start + index).into();
            Some(i.into())
        } else {
            None
        }
    }

    #[inline]
    fn one(self) -> error::Result<Item> {
        match self.len() {
            0 => Err(error::Error::XPTY0004),
            1 => {
                let i: IBig = self.start.into();
                Ok(i.into())
            }
            _ => Err(error::Error::XPTY0004),
        }
    }

    #[inline]
    fn option(self) -> error::Result<Option<Item>> {
        match self.len() {
            0 => Ok(None),
            1 => {
                let i: IBig = self.start.into();
                Ok(Some(i.into()))
            }
            _ => Err(error::Error::XPTY0004),
        }
    }

    #[inline]
    fn iter(&'a self) -> RangeIterator {
        RangeIterator {
            start: self.start,
            end: self.end,
            index: 0,
        }
    }

    #[inline]
    fn effective_boolean_value(&self) -> error::Result<bool> {
        match self.len() {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(error::Error::FORG0006),
        }
    }

    #[inline]
    fn string_value(&self, _xot: &xot::Xot) -> error::Result<String> {
        match self.len() {
            0 => Ok(String::new()),
            1 => {
                let i: IBig = self.start.into();
                Ok(i.to_string())
            }
            _ => Err(error::Error::XPTY0004),
        }
    }
}

pub struct RangeIterator {
    start: usize,
    end: usize,
    index: usize,
}

impl Iterator for RangeIterator {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < (self.end - self.start) {
            let i: IBig = (self.start + self.index).into();
            self.index += 1;
            Some(i.into())
        } else {
            None
        }
    }
}

// specifically implement the extensions for each version, so that
// we can avoid dynamic dispatch on the inside. We can't do it generically
// as we want a specialized version for the StackSequence so we can avoid
// dynamic dispatch on the inside.
impl<'a, I> SequenceExt<'a, I> for Empty
where
    I: Iterator<Item = Item> + 'a,
    Empty: SequenceCore<'a, I>,
{
    fn atomized(
        &'a self,
        _xot: &'a xot::Xot,
    ) -> impl Iterator<Item = error::Result<atomic::Atomic>> + 'a {
        std::iter::empty()
    }
}

impl<'a, I> SequenceCompare<'a, I> for Empty
where
    I: Iterator<Item = Item> + 'a,
    Empty: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for Empty
where
    I: Iterator<Item = Item>,
    Empty: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for One
where
    I: Iterator<Item = Item> + 'a,
    One: SequenceCore<'a, I>,
{
    fn atomized(
        &'a self,
        xot: &'a xot::Xot,
    ) -> impl Iterator<Item = error::Result<atomic::Atomic>> + 'a {
        AtomizedItemIter::new(&self.item, xot)
    }
}

impl<'a, I> SequenceCompare<'a, I> for One
where
    I: Iterator<Item = Item> + 'a,
    One: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for One
where
    I: Iterator<Item = Item>,
    One: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for Many
where
    I: Iterator<Item = Item> + 'a,
    Many: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceCompare<'a, I> for Many
where
    I: Iterator<Item = Item> + 'a,
    Many: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for Many
where
    I: Iterator<Item = Item>,
    Many: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceExt<'a, I> for Range
where
    I: Iterator<Item = Item> + 'a,
    Range: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceCompare<'a, I> for Range
where
    I: Iterator<Item = Item> + 'a,
    Range: SequenceCore<'a, I>,
{
}

impl<'a, I> SequenceOrder<'a, I> for Range
where
    I: Iterator<Item = Item>,
    Range: SequenceCore<'a, I>,
{
}
