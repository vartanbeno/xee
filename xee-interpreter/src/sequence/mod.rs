/// A sequence is a list of items, where each item is either a atomic value,
/// a node or a function. XPath is defined around sequences.
///
/// A sequence is a wrapper around a stack value, implemented by the
/// stack module.
mod item;
mod matching;
mod normalization;
mod opc;
mod sequence_core;
mod serialization;

pub use crate::stack::AtomizedIter;
pub(crate) use item::AtomizedItemIter;
pub use item::Item;
pub(crate) use opc::OptionParameterConverter;
pub use sequence_core::Sequence;
pub use sequence_core::{ItemIter, NodeIter};
pub(crate) use serialization::SerializationParameters;
