use xot::Xot;

use crate::error;
use crate::output;
use crate::output::item::{StackItem, StackValue};
use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    stack_value: stack::Value,
}

impl Sequence {
    pub(crate) fn new(stack_value: stack::Value) -> Self {
        Self { stack_value }
    }

    pub fn from_items(items: &[output::Item]) -> Self {
        if items.is_empty() {
            return Self {
                stack_value: stack::Value::Atomic(stack::Atomic::Empty),
            };
        }
        if items.len() == 1 {
            return Self {
                stack_value: items[0].clone().into_stack_value(),
            };
        }
        let stack_items = items
            .iter()
            .map(|item| item.to_stack_item())
            .collect::<Vec<_>>();
        Self {
            stack_value: stack::Value::Sequence(stack::Sequence::from_items(&stack_items)),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => true,
            stack::Value::Sequence(sequence) => sequence.borrow().is_empty(),
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => 0,
            stack::Value::Atomic(_) => 1,
            stack::Value::Sequence(sequence) => sequence.borrow().len(),
            stack::Value::Node(_) => 1,
            stack::Value::Closure(_) => 1,
            stack::Value::Step(_) => 1,
        }
    }

    pub fn iter(&self) -> SequenceIter {
        SequenceIter {
            stack_value: self.stack_value.clone(),
            index: 0,
        }
    }

    pub fn one(&self) -> error::Result<output::Item> {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => Err(error::Error::XPTY0004A),
            stack::Value::Atomic(_)
            | stack::Value::Node(_)
            | stack::Value::Closure(_)
            | stack::Value::Step(_) => Ok(output::Item::StackValue(StackValue(
                self.stack_value.clone(),
            ))),
            stack::Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                if sequence.len() != 1 {
                    return Err(error::Error::XPTY0004A);
                }
                Ok(output::Item::StackItem(StackItem(
                    (sequence.as_slice()[0]).clone(),
                )))
            }
        }
    }

    pub fn option(&self) -> error::Result<Option<output::Item>> {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => Ok(None),
            stack::Value::Atomic(_)
            | stack::Value::Node(_)
            | stack::Value::Closure(_)
            | stack::Value::Step(_) => Ok(Some(output::Item::StackValue(StackValue(
                self.stack_value.clone(),
            )))),
            stack::Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                if sequence.is_empty() {
                    return Ok(None);
                }
                if sequence.len() > 1 {
                    return Err(error::Error::XPTY0004A);
                }
                Ok(Some(output::Item::StackItem(StackItem(
                    (sequence.as_slice()[0]).clone(),
                ))))
            }
        }
    }
    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        match self.stack_value.effective_boolean_value() {
            Ok(b) => Ok(b),
            // TODO should handle errors better here
            Err(_) => Err(crate::Error::FORG0006),
        }
    }

    // pub fn atomized<'a>(&self, xot: &'a Xot) -> impl Iterator<Item = output::Atomic> + 'a {
    //     AtomizedIter {
    //         atomized_iter: self.stack_value.atomized(xot),
    //         xot,
    //     }
    // }
    pub fn atomized<'a>(&self, xot: &'a Xot) -> AtomizedIter<'a> {
        AtomizedIter {
            atomized_iter: self.stack_value.atomized(xot),
            xot,
        }
    }

    pub fn atomized_sequence(&self, xot: &Xot) -> error::Result<Sequence> {
        // TODO: conceivably we don't consume the iterator here
        let items = self
            .atomized(xot)
            .map(|a| {
                a.map(output::Item::from_atomic)
                    .map_err(|e| error::Error::XPTY0004A)
            })
            .collect::<error::Result<Vec<_>>>()?;
        Ok(Sequence::from_items(&items))
    }

    pub fn generalized_atomic<'a, T>(
        &self,
        xot: &'a Xot,
        extract: impl Fn(&output::Atomic) -> Option<T>,
    ) -> GeneralizedAtomicIter<'a, impl Fn(&output::Atomic) -> Option<T>> {
        GeneralizedAtomicIter {
            atomized_iter: self.atomized(xot),
            extract,
        }
    }

    pub fn one_atom(&self, xot: &Xot) -> error::Result<output::Atomic> {
        let mut atomized = self.atomized(xot);
        if let Some(one) = atomized.next() {
            if atomized.next().is_none() {
                Ok(one?)
            } else {
                Err(error::Error::XPTY0004A)
            }
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn option_atom(&self, xot: &Xot) -> error::Result<Option<output::Atomic>> {
        let mut atomized = self.atomized(xot);
        if let Some(one) = atomized.next() {
            if atomized.next().is_none() {
                Ok(Some(one?))
            } else {
                Err(error::Error::XPTY0004A)
            }
        } else {
            Ok(None)
        }
    }

    pub fn many_atom(&self, xot: &Xot) -> error::Result<Vec<output::Atomic>> {
        self.atomized(xot).collect::<Result<Vec<_>, _>>()
    }

    pub fn one_generalized_atomic<T, F>(&self, xot: &Xot, extract: F) -> error::Result<T>
    where
        F: Fn(&output::Atomic) -> error::Result<T>,
    {
        let atom = self.one_atom(xot)?;
        extract(&atom)
    }
}

pub struct SequenceIter {
    stack_value: stack::Value,
    index: usize,
}

impl Iterator for SequenceIter {
    type Item = output::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => None,
            stack::Value::Atomic(_)
            | stack::Value::Node(_)
            | stack::Value::Closure(_)
            | stack::Value::Step(_) => {
                if self.index == 0 {
                    self.index += 1;
                    Some(output::Item::StackValue(StackValue(
                        self.stack_value.clone(),
                    )))
                } else {
                    None
                }
            }
            stack::Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                if self.index < sequence.len() {
                    let item = sequence.items[self.index].clone();
                    self.index += 1;
                    Some(output::Item::StackItem(StackItem(item)))
                } else {
                    None
                }
            }
        }
    }
}

pub struct AtomizedIter<'a> {
    atomized_iter: stack::AtomizedIter<'a>,
    xot: &'a Xot,
}

impl<'a> Iterator for AtomizedIter<'a> {
    type Item = error::Result<output::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        self.atomized_iter.next().map(|a| {
            a.map(output::Atomic::new)
                .map_err(|e| error::Error::XPTY0004A)
        })
    }
}

pub struct GeneralizedAtomicIter<'a, F> {
    atomized_iter: output::AtomizedIter<'a>,
    extract: F,
}

// impl<'a, T, F> Iterator for GeneralizedAtomicIter<'a, F>
// where
//     F: Fn(&output::Atomic) -> Option<T>,
// {
//     type Item = T;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(atomic) = self.atomized_iter.next() {
//             (self.extract)(&atomic)
//         } else {
//             None
//         }
//     }
// }
