//! Error handling

pub use crate::documents::DocumentsError;
pub use xee_interpreter::error::{
    Error as ErrorValue, SpannedError as Error, SpannedResult as Result,
};
