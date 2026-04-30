use crate::errors::{GraphError, Result};
use crate::projector::GraphProjector;
use crate::vault::GraphVault;
use ai_brains_store::EventStore;

pub struct GraphRebuilder<'a> {
    vault: &'a GraphVault,
    store: &'a dyn EventStore,
}

impl<'a> GraphRebuilder<'a> {
    pub fn new(vault: &'a GraphVault, store: &'a dyn EventStore) -> Self {
        Self { vault, store }
    }

    pub fn rebuild(&self) -> Result<()> {
        let conn = self
            .vault
            .connection()
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        // Clear existing graph data
        conn.execute("DELETE FROM graph_edge", [])
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        conn.execute("DELETE FROM graph_node", [])
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        drop(conn); // Release lock before projector starts

        let projector = GraphProjector::new(self.vault);
        let events = self
            .store
            .read_all_events()
            .map_err(|e| GraphError::StoreError(e.to_string()))?;

        for envelope in events {
            projector.apply(&envelope)?;
        }

        Ok(())
    }
}
