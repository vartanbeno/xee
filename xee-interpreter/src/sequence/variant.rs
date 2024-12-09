use std::rc::Rc;

use ibig::IBig;
use xot::Xot;

use crate::atomic::AtomicCompareValue;
use crate::{atomic, error};

use super::item::Item;
use super::traits::{SequenceCompare, SequenceCore, SequenceExt, SequenceOrder};
use super::AtomizedItemIter;

// this size should be below a usize
const MAXIMUM_RANGE_SIZE: i64 = 2_i64.pow(25);

#[derive(Debug, Clone, PartialEq)]
pub struct Empty {}

impl SequenceCore<'_, std::iter::Empty<Item>> for Empty {
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
    start: Box<IBig>,
    end: Box<IBig>,
}

impl Range {
    pub(crate) fn new(start: IBig, end: IBig) -> error::Result<Self> {
        let length: IBig = &end - &start;
        if length > MAXIMUM_RANGE_SIZE.into() {
            return Err(error::Error::FOAR0002);
        }

        Ok(Range {
            start: start.into(),
            end: end.into(),
        })
    }

    pub(crate) fn start(&self) -> &IBig {
        &self.start
    }
    pub(crate) fn end(&self) -> &IBig {
        &self.end
    }

    pub(crate) fn contains(&self, index: &IBig) -> bool {
        index >= self.start.as_ref() && index < self.end.as_ref()
    }

    pub(crate) fn general_comparison_integer(
        &self,
        value: &IBig,
        comparison: AtomicCompareValue,
    ) -> bool {
        match comparison {
            // value has to be within the range
            AtomicCompareValue::Eq => value >= self.start.as_ref() && value < self.end.as_ref(),
            // value has to be outside the range
            AtomicCompareValue::Ne => value < self.start.as_ref() || value >= self.end.as_ref(),
            // value has to be greater than the start
            // 10 gt 10..11 is false
            AtomicCompareValue::Gt => value > self.start.as_ref(),
            // value has to be less than the end - 1
            // 10 lt 10..11 is false
            AtomicCompareValue::Lt => {
                let one: IBig = 1.into();
                let end = self.end.as_ref() - &one;
                value < &end
            }
            // value has to be greater than or equal to the start
            // 10 ge 10..11 is true
            AtomicCompareValue::Ge => value >= self.start.as_ref(),
            // value has to be less than the end
            // 10 le 10..11 is true
            AtomicCompareValue::Le => value < self.end.as_ref(),
        }
    }
}

impl<'a> SequenceCore<'a, RangeIterator> for Range {
    #[inline]
    fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    fn len(&self) -> usize {
        let len = self.end.as_ref() - self.start.as_ref();
        // We should prevent any range that's > usize from being crated
        len.try_into().unwrap()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<Item> {
        if index < self.len() {
            let i: IBig = self.start.as_ref() + index;
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
                let i: IBig = self.start.as_ref().clone();
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
                let i: IBig = self.start.as_ref().clone();
                Ok(Some(i.into()))
            }
            _ => Err(error::Error::XPTY0004),
        }
    }

    #[inline]
    fn iter(&'a self) -> RangeIterator {
        RangeIterator {
            start: self.start.as_ref().clone(),
            end: self.end.as_ref().clone(),
            index: 0.into(),
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
            1 => Ok(self.start.to_string()),
            _ => Err(error::Error::XPTY0004),
        }
    }
}

pub struct RangeIterator {
    start: IBig,
    end: IBig,
    index: IBig,
}

impl RangeIterator {
    pub(crate) fn new(start: IBig, end: IBig) -> Self {
        RangeIterator {
            start,
            end,
            index: 0.into(),
        }
    }
}

impl Iterator for RangeIterator {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < (&self.end - &self.start) {
            let i = &self.start + &self.index;
            self.index += 1;
            Some(i.into())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = &self.end - &self.start;
        // we know that we don't have a range that's > usize as we cannot construct
        // any
        let len: usize = len.try_into().expect("range size is within usize");
        (len, Some(len))
    }
}

// We implement specific interfaces here, instead of generically for all
// T that implement sequenceCore, because we want to provide a few specialized
// implementations.
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

    /// Get just one atomized value from the sequence
    fn atomized_one(&'a self, _xot: &'a Xot) -> error::Result<atomic::Atomic> {
        Err(error::Error::XPTY0004)
    }

    /// Get an optional atomized value from the sequence
    fn atomized_option(&'a self, _xot: &'a Xot) -> error::Result<Option<atomic::Atomic>> {
        Ok(None)
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
