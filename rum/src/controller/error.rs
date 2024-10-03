use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("database error: {0}")]
    OrmError(#[from] crate::model::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("fmt error: {0}")]
    FmtError(#[from] std::fmt::Error),

    #[error("comms error: {0}")]
    CommsError(#[from] crate::comms::Error),

    #[error("{0}")]
    Error(#[from] Box<dyn std::error::Error + Sync + Send>),
}

impl Error {
    /// Create new error from any error implementing the standard [`Error`] trait.
    pub fn new(err: impl std::error::Error + Send + Sync + 'static) -> Error {
        Error::Error(Box::new(err))
    }
}

impl From<crate::http::Error> for Error {
    fn from(error: crate::http::Error) -> Self {
        Error::new(Box::new(error))
    }
}
