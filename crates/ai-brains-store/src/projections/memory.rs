use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct MemoryProjection;

impl Projection for MemoryProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
        let privacy = serde_json::to_string(&envelope.privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::MemoryPinned(p) => {
                tx.execute(
                    "INSERT INTO memory_projection (memory_id, session_id, content, privacy, status, level, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        session_id = COALESCE(excluded.session_id, memory_projection.session_id),
                        status = excluded.status,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        p.session_id.as_ref().map(|s| s.to_string()),
                        p.content,
                        privacy,
                        "pinned",
                        0, // Level 0 for pinned memories
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            Payload::SessionSummaryCreated(p) => {
                tx.execute(
                    "INSERT INTO memory_projection (memory_id, session_id, content, privacy, status, level, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        session_id = COALESCE(excluded.session_id, memory_projection.session_id),
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        p.session_id.to_string(),
                        p.summary,
                        privacy,
                        "pinned",
                        0, // Level 0 for session summaries
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            Payload::MemorySynthesized(p) => {
                tx.execute(
                    "INSERT INTO memory_projection (memory_id, project_id, content, privacy, status, level, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        level = excluded.level,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        p.project_id.to_string(),
                        p.content,
                        privacy,
                        "pinned",
                        p.level,
                        occurred_at,
                        occurred_at
                    ],
                )?;

                // Track hierarchy
                for source_id in &p.source_memory_ids {
                    tx.execute(
                        "INSERT INTO memory_hierarchy (parent_memory_id, child_memory_id) VALUES (?, ?)
                         ON CONFLICT DO NOTHING",
                        rusqlite::params![p.memory_id.to_string(), source_id.to_string()],
                    )?;
                }
            }
            Payload::MemoryForgotten(p) => {
                tx.execute(
                    "UPDATE memory_projection SET status = ?, updated_at = ? WHERE memory_id = ?",
                    rusqlite::params!["forgotten", occurred_at, p.memory_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
