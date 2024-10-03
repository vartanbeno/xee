//! Sequence items.
//!
//! An item is either an [`Atomic`] value, a [`xot::Node`] or a
//! function item.
pub use xee_interpreter::atomic::Atomic;
pub use xee_interpreter::function::{Array, Function, Map};
pub use xee_interpreter::sequence::Item;
