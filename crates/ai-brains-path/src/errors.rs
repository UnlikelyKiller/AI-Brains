use thiserror::Error;

pub type Result<T> = std::result::Result<T, PathError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathError {
    #[error("path input is empty")]
    EmptyInput,
    #[error("path contains a NUL byte")]
    NulByte,
    #[error("relative paths are not supported for canonical project identity: {0}")]
    RelativePath(String),
    #[error("malformed WSL mount path: {0}")]
    MalformedWslPath(String),
    #[error("I/O error during path normalization: {0}")]
    IoError(String),
}
