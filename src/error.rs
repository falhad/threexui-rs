use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
