/// XSLT has template rules that match based on patterns, a subset of XPath.
/// This module contains the runtime to match items with patterns.
mod pattern_core;
mod pattern_core2;

pub use pattern_core::PatternLookup;
