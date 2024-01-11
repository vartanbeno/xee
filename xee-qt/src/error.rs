use std::path::PathBuf;
use thiserror::Error;

use crate::outcome;
use crate::qt;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Test failures {0} {1}")]
    TestFailures(PathBuf, outcome::TestSetOutcomes),
    #[error("catalog.xml cannot be found")]
    NoCatalogFound,
    #[error("File not found in catalog: {0}")]
    FileNotFoundInCatalog(PathBuf),
    #[error("Unknown environment reference")]
    UnknownEnvironmentReference(qt::EnvironmentRef),
    #[error("Cannot represent as XML")]
    CannotRepresentAsXml,
    #[error("Xee XPath error")]
    XeeXPath(#[from] xee_xpath::error::Error),
    #[error("Xot error")]
    Xot(#[from] xot::Error),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("Var error")]
    VarError(#[from] std::env::VarError),
    #[error("Globset error")]
    GlobSet(#[from] globset::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
