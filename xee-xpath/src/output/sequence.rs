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

    pub fn one(&self) -> stack::Result<output::Item> {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => Err(stack::Error::Type),
            stack::Value::Atomic(_)
            | stack::Value::Node(_)
            | stack::Value::Closure(_)
            | stack::Value::Step(_) => Ok(output::Item::StackValue(StackValue(
                self.stack_value.clone(),
            ))),
            stack::Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                if sequence.len() != 1 {
                    return Err(stack::Error::Type);
                }
                Ok(output::Item::StackItem(StackItem(
                    (sequence.as_slice()[0]).clone(),
                )))
            }
        }
    }

    pub fn option(&self) -> stack::Result<Option<output::Item>> {
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
                    return Err(stack::Error::Type);
                }
                Ok(Some(output::Item::StackItem(StackItem(
                    (sequence.as_slice()[0]).clone(),
                ))))
            }
        }
    }
    pub fn effective_boolean_value(&self) -> std::result::Result<bool, crate::error::Error> {
        match self.stack_value.effective_boolean_value() {
            Ok(b) => Ok(b),
            // TODO should handle errors better here
            Err(_) => Err(crate::Error::FORG0006),
        }
    }

    // pub fn items(&self) -> &[output::Item] {
    //     match self.stack_value {
    //         stack::Value::Atomic(stack::Atomic::Empty) => &[],
    //         stack::Value::Sequence(sequence) => sequence.borrow().as_slice(),
    //         _ => unreachable!("Not a sequence"),
    //     }
    // }

    // XXX unfortunate duplication with effective_boolean_value
    // on Value
    // pub fn effective_boolean_value(&self) -> std::result::Result<bool, crate::error::Error> {
    //     if self.items.is_empty() {
    //         return Ok(false);
    //     }
    //     if matches!(self.items[0], output::Item::Node(_)) {
    //         return Ok(true);
    //     }
    //     if self.items.len() != 1 {
    //         return Err(crate::Error::FORG0006);
    //     }
    //     match self.items[0].to_bool() {
    //         Ok(b) => Ok(b),
    //         Err(_) => Err(crate::Error::FORG0006),
    //     }
    // }
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
