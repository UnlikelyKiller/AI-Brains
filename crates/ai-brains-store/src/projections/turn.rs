use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct TurnProjection;

impl Projection for TurnProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        let (session_id, role, content, tx_id) = match &envelope.payload {
            Payload::UserPromptRecorded(p) => (
                p.session_id.to_string(),
                "user",
                p.content.clone(),
                p.tx_id.as_ref().map(|t| t.to_string()),
            ),
            Payload::AssistantFinalRecorded(p) => (
                p.session_id.to_string(),
                "assistant",
                p.content.clone(),
                p.tx_id.as_ref().map(|t| t.to_string()),
            ),
            _ => return Ok(()),
        };

        // Fetch project_id from session_projection
        let project_id: String = tx.query_row(
            "SELECT project_id FROM session_projection WHERE session_id = ?",
            [&session_id],
            |row| row.get(0),
        )?;

        let turn_index: i64 = tx.query_row(
            "SELECT COALESCE(MAX(turn_index), -1) + 1 FROM turn_projection WHERE session_id = ?",
            [&session_id],
            |row| row.get(0),
        )?;

        tx.execute(
            "INSERT INTO turn_projection (session_id, project_id, turn_index, role, content, tx_id, occurred_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                session_id,
                project_id,
                turn_index,
                role,
                content,
                tx_id,
                occurred_at
            ],
        )?;

        // Also project into memory for lexical search (recall)
        let memory_id = ai_brains_core::ids::MemoryId::new();
        let privacy = serde_json::to_string(&envelope.privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        tx.execute(
            "INSERT INTO memory_projection (memory_id, session_id, project_id, content, privacy, status, level, tx_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(memory_id) DO UPDATE SET
                content = excluded.content,
                tx_id = COALESCE(excluded.tx_id, memory_projection.tx_id),
                updated_at = excluded.updated_at",
            rusqlite::params![
                memory_id.to_string(),
                session_id,
                project_id,
                format!("{}: {}", role.to_uppercase(), content),
                privacy,
                "pinned", // Mark as pinned so it's searchable by default
                0,
                tx_id,
                occurred_at,
                occurred_at
            ],
        )?;

        Ok(())
    }
}
