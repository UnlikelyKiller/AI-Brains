use crate::cozo_proxy::CozoProxyBackend;
use crate::errors::{GraphError, Result};
use crate::multiplex::MultiplexGraphBackend;
use crate::projector::GraphProjector;
use crate::sqlite_backend::SqliteGraphBackend;
use crate::vault::GraphVault;
use ai_brains_store::EventStore;
use std::sync::Arc;

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

        // Clear existing local graph data
        conn.execute("DELETE FROM graph_edge", [])
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        conn.execute("DELETE FROM graph_node", [])
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        drop(conn); // Release lock

        // Setup multiplexer
        let sqlite_backend = Box::new(SqliteGraphBackend::new(Arc::new(
            self.vault.connection().clone(),
        )));
        let cozo_backend = Box::new(CozoProxyBackend::new(None));

        let multiplexer: Box<dyn crate::cozo_proxy::GraphBackend + Send + Sync> =
            Box::new(MultiplexGraphBackend::new(vec![
                sqlite_backend,
                cozo_backend,
            ]));

        let mut projector = GraphProjector::new(multiplexer);
        let events = self
            .store
            .read_all_events()
            .map_err(|e| GraphError::StoreError(e.to_string()))?;

        for envelope in events {
            projector.apply(&envelope)?;
        }

        projector.flush()?;

        Ok(())
    }
}
