use thiserror::Error;

pub type Result<T> = std::result::Result<T, CaptureError>;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("unsupported role: {0}")]
    UnsupportedRole(String),
    #[error("user prompt content cannot be empty")]
    EmptyPrompt,
    #[error("assistant final content cannot be empty unless status-only")]
    EmptyFinal,
    #[error("session stop reason is required for failed status")]
    MissingFailureReason,
    #[error("event build failed: {0}")]
    Event(#[from] ai_brains_events::EventError),
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("git metadata failed: {0}")]
    Git(#[from] ai_brains_git::GitError),
    #[error("validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", "))]
    ValidationErrors(Vec<ValidationError>),
}
