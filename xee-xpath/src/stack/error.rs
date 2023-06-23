use miette::Diagnostic;
use thiserror::Error;

pub use crate::error::{Error, Result};

// #[derive(Debug, Error, Diagnostic, Clone, PartialEq)]
// pub(crate) enum Error {
//     #[error("Type error")]
//     XPTY0004,
//     #[error("Type error")]
//     Type,
//     #[error("Overflow/underflow")]
//     Overflow,
//     #[error("Division by zero")]
//     DivisionByZero,
//     #[error("Stack overflow")]
//     StackOverflow,
//     #[error("Absent")]
//     Absent,
//     // Explicit error raised with Error
//     #[error("Error")]
//     Error(#[from] XeeError),
// }

// pub(crate) type Result<T> = std::result::Result<T, Error>;
