use miette::Diagnostic;
use thiserror::Error;

use crate::error::Error;

#[derive(Debug, Error, Diagnostic, Clone, PartialEq)]
pub enum ValueError {
    #[error("Type error")]
    XPTY0004,
    #[error("Type error")]
    Type,
    #[error("Overflow/underflow")]
    Overflow,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Absent")]
    Absent,
    // Explicit error raised with Error
    #[error("Error")]
    Error(Error),
}

type Result<T> = std::result::Result<T, ValueError>;
