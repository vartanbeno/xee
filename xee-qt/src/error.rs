use thiserror::Error;

use crate::qt;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown environment reference")]
    UnknownEnvironmentReference(qt::EnvironmentRef),
    #[error("Xee XPath error")]
    XeeXPath(xee_xpath::Error),
    #[error("Xot error")]
    Xot(xot::Error),
    #[error("IO error")]
    IO(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// turn any IO error into Error
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}

// turn any Xot error into Error
impl From<xot::Error> for Error {
    fn from(err: xot::Error) -> Self {
        Error::Xot(err)
    }
}

// turn any Xee XPath error into Error
impl From<xee_xpath::Error> for Error {
    fn from(err: xee_xpath::Error) -> Self {
        Error::XeeXPath(err)
    }
}
