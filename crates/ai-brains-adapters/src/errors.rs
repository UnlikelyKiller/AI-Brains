use thiserror::Error;

pub type Result<T> = std::result::Result<T, AdapterError>;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("store error: {0}")]
    Store(#[from] rusqlite::Error),
    #[error("capture error: {0}")]
    Capture(#[from] ai_brains_capture::CaptureError),
    #[error("{0}")]
    Other(String),
}
