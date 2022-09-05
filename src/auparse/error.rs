use std::time::SystemTimeError;
use thiserror::Error;

/// An error that can occur
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to init auparse")]
    NativeInitFail,

    #[error("{0}")]
    DurationError(#[from] SystemTimeError),

    #[error("General failure: {0}")]
    GeneralFail(String),
}
