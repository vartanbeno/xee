// the design goals of this module is to provide an efficient Sequence representation.

// Goals:
// - Sequence should be relatively small in memory size.
// - Optimized versions of special cases: empty sequence and sequence of only one value
//
// To this end dynamic dispatch (Box<dyn>) is used only to implement the outer
// iterators. This should allow the inner iteration to get compiled away for the
// empty and one case.

mod compare;
mod comparison;
mod core;
mod creation;
mod iter;
mod matching;
mod traits;
mod variant;
