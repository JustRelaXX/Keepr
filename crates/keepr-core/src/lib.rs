pub mod dates;
pub mod model;
pub mod store;

pub use model::*;
pub use store::Store;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Validation(&'static str),
    #[error("not_found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error("database: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("data: {0}")]
    Data(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(code) => code,
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            _ => "storage_error",
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
