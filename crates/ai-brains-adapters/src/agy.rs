use crate::errors::{AdapterError, Result};
use ai_brains_core::ids::{SessionId, TurnId};
use serde::Deserialize;
use std::path::Path;
use uuid::Uuid;

/// A single line from the agy JSONL transcript.
#[derive(Debug, Clone, Deserialize)]
pub struct AgyTranscriptLine {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

pub struct AgyTurn {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

pub fn parse_agy_transcript(path: &Path) -> Result<Vec<AgyTurn>> {
    let content = std::fs::read_to_string(path)?;

    let mut turns = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let transcript_line: AgyTranscriptLine = serde_json::from_str(line)
            .map_err(|e| AdapterError::Other(format!("Failed to parse agy JSONL: {}", e)))?;

        turns.push(AgyTurn {
            role: transcript_line.role,
            content: transcript_line.content,
            timestamp: transcript_line.timestamp,
        });
    }

    Ok(turns)
}

pub fn generate_deterministic_turn_id(session_id: &SessionId, index: usize) -> TurnId {
    TurnId::from_uuid(Uuid::new_v5(
        &session_id.as_uuid(),
        format!("agy-turn-{}", index).as_bytes(),
    ))
}
