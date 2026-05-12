use crate::errors::{CaptureError, Result, ValidationError};
use ai_brains_contracts::ingest::IngestRequest;
use serde_json::Value;

pub fn parse_ingest_request(json: &str) -> Result<IngestRequest> {
    let value: Value = serde_json::from_str(json).map_err(CaptureError::Json)?;
    let mut errors: Vec<ValidationError> = Vec::new();

    let required_fields = [
        "session_id",
        "project_id",
        "harness_id",
        "turn_id",
        "role",
        "content",
        "privacy",
    ];

    for field in &required_fields {
        match value.get(field) {
            None => errors.push(ValidationError {
                field: field.to_string(),
                message: "missing required field".into(),
            }),
            Some(Value::Null) => errors.push(ValidationError {
                field: field.to_string(),
                message: "cannot be null".into(),
            }),
            Some(v) if v.is_string() && v.as_str().is_some_and(|s| s.trim().is_empty()) => {
                errors.push(ValidationError {
                    field: field.to_string(),
                    message: "cannot be empty string".into(),
                });
            }
            _ => {}
        }
    }

    if let Some(role) = value.get("role").and_then(|v| v.as_str()) {
        if !["user", "assistant", "system"].contains(&role) {
            errors.push(ValidationError {
                field: "role".into(),
                message: format!("unsupported role '{}'", role),
            });
        }
    }

    if !errors.is_empty() {
        return Err(CaptureError::ValidationErrors(errors));
    }

    let request: IngestRequest = serde_json::from_value(value).map_err(CaptureError::Json)?;
    Ok(request)
}
