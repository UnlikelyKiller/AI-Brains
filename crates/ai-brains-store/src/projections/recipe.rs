use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct RecipeProjection;

impl Projection for RecipeProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        if let Payload::RecipePromoted(p) = &envelope.payload {
            let steps_json = serde_json::to_string(&p.steps)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            tx.execute(
                "INSERT INTO recipe_projection (recipe_id, name, steps_json, created_at)
                 VALUES (?, ?, ?, ?)",
                rusqlite::params![p.recipe_id.to_string(), p.name, steps_json, occurred_at],
            )?;
        }

        Ok(())
    }
}
