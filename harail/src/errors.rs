use thiserror::Error;
#[derive(Error, Debug)]
pub enum HaError {
    #[error("Incorrect usage: {0}")]
    UsageError(String),
    #[error("GTFS parse failed: {0}")]
    GTFSError(String),
}
