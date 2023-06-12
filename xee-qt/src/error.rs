use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

use crate::assert;
use crate::qt;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error("Test failures")]
    #[diagnostic()]
    TestFailures(PathBuf, Vec<assert::TestOutcome>),
    #[error("catalog.xml cannot be found")]
    #[diagnostic()]
    NoCatalogFound,
    #[error("Unknown environment reference")]
    #[diagnostic()]
    UnknownEnvironmentReference(qt::EnvironmentRef),
    #[error("Xee XPath error")]
    #[diagnostic()]
    XeeXPath(#[from] xee_xpath::Error),
    #[error("Xot error")]
    #[diagnostic()]
    Xot(#[from] xot::Error),
    #[error("IO error")]
    #[diagnostic()]
    IO(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// impl<T> IntoDiagnostic<T, Error> for std::result::Result<T, Error> {}

// // turn any IO error into Error
// impl From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Self {
//         Error::IO(err)
//     }
// }

// // turn any Xot error into Error
// impl From<xot::Error> for Error {
//     fn from(err: xot::Error) -> Self {
//         Error::Xot(err)
//     }
// }

// // turn any Xee XPath error into Error
// impl From<xee_xpath::Error> for Error {
//     fn from(err: xee_xpath::Error) -> Self {
//         Error::XeeXPath(err)
//     }
// }
