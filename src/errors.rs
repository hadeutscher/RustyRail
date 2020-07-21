use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Incorrect usage: {0}")]
    UsageError(String),
}
