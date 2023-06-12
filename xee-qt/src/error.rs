use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Xee XPath error")]
    XeeXPathError(xee_xpath::Error),
    #[error("Xot error")]
    XotError(xot::Error),
    #[error("IO error")]
    IOError(std::io::Error),
}

// turn any IO error into Error
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

// turn any Xot error into Error
impl From<xot::Error> for Error {
    fn from(err: xot::Error) -> Self {
        Error::XotError(err)
    }
}

// turn any Xee XPath error into Error
impl From<xee_xpath::Error> for Error {
    fn from(err: xee_xpath::Error) -> Self {
        Error::XeeXPathError(err)
    }
}
