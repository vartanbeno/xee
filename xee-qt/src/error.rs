use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

use crate::assert;
use crate::qt;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error("Test failures {0} {1}")]
    #[diagnostic()]
    TestFailures(PathBuf, assert::TestOutcomes),
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
    #[error("Var error")]
    #[diagnostic()]
    VarError(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, Error>;
