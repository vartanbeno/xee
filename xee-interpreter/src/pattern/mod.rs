// XSLT has template rules that match based on patterns, a subset of XPath.
// This module contains the runtime to match items with patterns.

mod lookup;
mod pattern_core;

pub use lookup::ModeLookup;
pub(crate) use pattern_core::PredicateMatcher;
