/// A sequence is a list of items, where each item is either a atomic value,
/// a node or a function. XPath is defined around sequences.
///
/// A sequence is a wrapper around a stack value, implemented by the
/// stack module.
mod matching;
mod normalization;
mod opc;
mod sequence_core;
mod serialization;

pub(crate) use crate::neovalue::AtomizedItemIter;
pub use crate::neovalue::Item;
pub use crate::stack::AtomizedIter;
pub(crate) use opc::OptionParameterConverter;
pub use sequence_core::Sequence;
pub use sequence_core::{ItemIter, NodeIter};
pub(crate) use serialization::SerializationParameters;
