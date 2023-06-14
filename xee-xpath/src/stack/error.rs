use miette::Diagnostic;
use thiserror::Error;

use crate::error::Error as XeeError;

#[derive(Debug, Error, Diagnostic, Clone, PartialEq)]
pub enum Error {
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
    Error(XeeError),
}

pub type Result<T> = std::result::Result<T, Error>;
