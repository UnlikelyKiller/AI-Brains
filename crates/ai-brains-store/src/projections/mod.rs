use crate::errors::Result;

use ai_brains_events::Envelope;
use rusqlite::Transaction;

pub mod conflict;
pub mod memory;
pub mod project;
pub mod recipe;
pub mod session;
pub mod turn;

pub trait Projection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()>;
}

pub fn apply_all(tx: &Transaction, envelope: &Envelope) -> Result<()> {
    project::ProjectProjection.apply(tx, envelope)?;
    session::SessionProjection.apply(tx, envelope)?;
    turn::TurnProjection.apply(tx, envelope)?;
    memory::MemoryProjection.apply(tx, envelope)?;
    conflict::ConflictProjection.apply(tx, envelope)?;
    recipe::RecipeProjection.apply(tx, envelope)?;
    Ok(())
}
