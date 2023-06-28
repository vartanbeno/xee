mod arithmetic;
mod atomic_core;
mod cast;
mod comparison;

pub use atomic_core::Atomic;
pub(crate) use comparison::{
    comparison_op, ComparisonOp, EqualOp, GreaterThanOp, GreaterThanOrEqualOp, LessThanOp,
    LessThanOrEqualOp, NotEqualOp,
};
