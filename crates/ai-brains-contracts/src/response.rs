use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub ok: bool,
    /// Compatibility field for external tools (e.g. ChangeGuard)
    pub status: String,
    /// Compatibility field for external tools (flat message)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<String>,
}

impl<T> ApiResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            status: "success".to_string(),
            message: None,
            data: Some(data),
            error: None,
            warnings: Vec::new(),
        }
    }

    pub fn error(error: ApiError) -> Self {
        let message = Some(error.message.clone());
        Self {
            ok: false,
            status: "error".to_string(),
            message,
            data: None,
            error: Some(error),
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
