use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Numeric operation overflow/underflow")]
    FOAR0002,
    #[error("type error")]
    TypeError,
}

pub type Result<T> = std::result::Result<T, Error>;
