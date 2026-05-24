use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct ProjectProjection;

impl Projection for ProjectProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::ProjectRegistered(p) => {
                tx.execute(
                    "INSERT INTO project_projection (project_id, name, tx_id, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?)
                     ON CONFLICT(project_id) DO UPDATE SET
                        name = excluded.name,
                        tx_id = COALESCE(excluded.tx_id, project_projection.tx_id),
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.project_id.to_string(),
                        p.name,
                        p.tx_id.as_ref().map(|t| t.to_string()),
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            Payload::ProjectAliasAdded(p) => {
                tx.execute(
                    "INSERT INTO project_alias_projection (alias, project_id) \
                     VALUES (?, ?) \
                     ON CONFLICT(alias) DO NOTHING",
                    rusqlite::params![p.alias, p.project_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
