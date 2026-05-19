use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{Envelope, Payload};
use rusqlite::{OptionalExtension, Transaction};
use time::format_description::well_known::Rfc3339;

pub struct SessionProjection;

impl Projection for SessionProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::SessionStarted(p) => {
                let privacy_json = serde_json::to_string(&envelope.privacy)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                tx.execute(
                    "INSERT INTO session_projection (session_id, project_id, status, privacy, tx_id, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(session_id) DO UPDATE SET
                        status = excluded.status,
                        tx_id = COALESCE(excluded.tx_id, session_projection.tx_id),
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.session_id.to_string(),
                        p.project_id.to_string(),
                        "active",
                        privacy_json,
                        p.tx_id.as_ref().map(|t| t.to_string()),
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            Payload::UserPromptRecorded(p) => {
                self.update_session_provenance(
                    tx,
                    &p.session_id.to_string(),
                    envelope.privacy,
                    p.tx_id.as_ref(),
                    &occurred_at,
                )?;
            }
            Payload::AssistantFinalRecorded(p) => {
                self.update_session_provenance(
                    tx,
                    &p.session_id.to_string(),
                    envelope.privacy,
                    p.tx_id.as_ref(),
                    &occurred_at,
                )?;
            }
            Payload::SessionCompleted(p) => {
                tx.execute(
                    "UPDATE session_projection SET status = ?, updated_at = ? WHERE session_id = ?",
                    rusqlite::params!["completed", occurred_at, p.session_id.to_string()],
                )?;
            }
            Payload::SessionFailed(p) => {
                tx.execute(
                    "UPDATE session_projection SET status = ?, updated_at = ? WHERE session_id = ?",
                    rusqlite::params!["failed", occurred_at, p.session_id.to_string()],
                )?;
            }
            Payload::SessionSummaryCreated(p) => {
                tx.execute(
                    "UPDATE session_projection 
                     SET summary_memory_id = ?, summarized_at = ?, updated_at = ? 
                     WHERE session_id = ?",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        occurred_at,
                        occurred_at,
                        p.session_id.to_string()
                    ],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl SessionProjection {
    fn update_session_provenance(
        &self,
        tx: &Transaction,
        session_id: &str,
        new_privacy: Privacy,
        tx_id: Option<&ai_brains_core::ids::TransactionId>,
        occurred_at: &str,
    ) -> Result<()> {
        // Handle Privacy Escalation (Strictest wins)
        let current_privacy_json: Option<String> = tx
            .query_row(
                "SELECT privacy FROM session_projection WHERE session_id = ?",
                rusqlite::params![session_id],
                |row| row.get(0),
            )
            .optional()?;

        let final_privacy = if let Some(json) = current_privacy_json {
            let current_privacy: Privacy = serde_json::from_str(&json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
            current_privacy.combine(new_privacy)
        } else {
            new_privacy
        };

        let final_privacy_json = serde_json::to_string(&final_privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        tx.execute(
            "UPDATE session_projection 
             SET privacy = ?, 
                 tx_id = COALESCE(?, tx_id),
                 updated_at = ? 
             WHERE session_id = ?",
            rusqlite::params![
                final_privacy_json,
                tx_id.map(|t| t.to_string()),
                occurred_at,
                session_id
            ],
        )?;

        Ok(())
    }
}
