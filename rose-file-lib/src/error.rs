use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoseLibError {
    #[error("Error: {:?}", (.0))]
    Generic(String),

    #[error("File error: {:?}", (path.display()))]
    FileError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl From<std::convert::Infallible> for RoseLibError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}
