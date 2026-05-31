//! Live graph hook: applies each appended event to the GraphProjector and flushes
//! to SQLite immediately. Graph failures are always non-fatal.

#[cfg(feature = "graph")]
use ai_brains_events::Envelope;
#[cfg(feature = "graph")]
use ai_brains_graph::{
    CozoProxyBackend, GraphProjector, MultiplexGraphBackend, SqliteGraphBackend,
};
#[cfg(feature = "graph")]
use ai_brains_store::VaultConnection;
#[cfg(feature = "graph")]
use std::sync::Arc;

/// Owns a GraphProjector backed by SQLite (+ CozoDB if ChangeGuard is available).
/// `apply_and_flush` is non-fatal: graph failures never block the primary append.
#[cfg(feature = "graph")]
pub struct LiveGraphHook {
    projector: GraphProjector<'static>,
}

#[cfg(feature = "graph")]
impl LiveGraphHook {
    pub fn new(conn: Arc<VaultConnection>) -> Self {
        let sqlite = Box::new(SqliteGraphBackend::new(conn));
        let cozo = Box::new(CozoProxyBackend::new(None));
        let multiplex: Box<dyn ai_brains_graph::GraphBackend + Send + Sync + 'static> =
            Box::new(MultiplexGraphBackend::new(vec![sqlite, cozo]));
        Self {
            projector: GraphProjector::new(multiplex),
        }
    }

    /// Apply a single event to the graph and flush immediately.
    /// Errors are logged as warnings and never propagated.
    pub fn apply_and_flush(&mut self, envelope: &Envelope) {
        if let Err(e) = self.projector.apply(envelope) {
            tracing::warn!("LiveGraphHook: apply failed: {}", e);
            return;
        }
        if let Err(e) = self.projector.flush() {
            tracing::warn!("LiveGraphHook: flush failed: {}", e);
        }
    }
}

/// Wraps SqliteEventStore and applies each appended event to LiveGraphHook.
/// Graph failures are non-fatal — primary store append always succeeds.
#[cfg(feature = "graph")]
pub struct GraphAwareEventStore {
    inner: ai_brains_store::SqliteEventStore,
    hook: std::sync::Mutex<LiveGraphHook>,
}

#[cfg(feature = "graph")]
impl GraphAwareEventStore {
    pub fn new(conn: VaultConnection) -> Self {
        let hook = LiveGraphHook::new(Arc::new(conn.clone()));
        Self {
            inner: ai_brains_store::SqliteEventStore::new(conn),
            hook: std::sync::Mutex::new(hook),
        }
    }
}

#[cfg(feature = "graph")]
impl ai_brains_store::EventStore for GraphAwareEventStore {
    fn append_event(&self, envelope: &Envelope) -> ai_brains_store::errors::Result<()> {
        self.inner.append_event(envelope)?;
        // Non-fatal graph update
        match self.hook.lock() {
            Ok(mut h) => h.apply_and_flush(envelope),
            Err(e) => tracing::warn!("LiveGraphHook: mutex poisoned: {}", e),
        }
        Ok(())
    }

    fn read_events(
        &self,
        aggregate_id: uuid::Uuid,
    ) -> ai_brains_store::errors::Result<Vec<Envelope>> {
        self.inner.read_events(aggregate_id)
    }

    fn read_all_events(&self) -> ai_brains_store::errors::Result<Vec<Envelope>> {
        self.inner.read_all_events()
    }

    fn get_sync_state(&self, key: &str) -> ai_brains_store::errors::Result<Option<String>> {
        self.inner.get_sync_state(key)
    }

    fn set_sync_state(&self, key: &str, value: &str) -> ai_brains_store::errors::Result<()> {
        self.inner.set_sync_state(key, value)
    }

    fn get_session_privacy(
        &self,
        session_id: &str,
    ) -> ai_brains_store::errors::Result<Option<ai_brains_core::privacy::Privacy>> {
        self.inner.get_session_privacy(session_id)
    }
}

#[cfg(all(test, feature = "graph"))]
mod tests {
    use super::*;
    use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
    use ai_brains_core::privacy::Privacy;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use ai_brains_events::{
        constructors::EventBuilder, Actor, AggregateType, EventKind, MemoryPinnedPayload, Payload,
    };
    use ai_brains_graph::{GraphSearch, GraphVault};
    use ai_brains_store::connection::VaultConnection;
    use ai_brains_store::EventStore;
    use tempfile::NamedTempFile;

    fn setup_connection() -> Result<VaultConnection, Box<dyn std::error::Error>> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file
            .path()
            .to_str()
            .ok_or("invalid temp path")?
            .to_string();
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        let conn = VaultConnection::open(&db_path, &sql_key)?;
        conn.migrate()?;
        Ok(conn)
    }

    #[test]
    fn graph_aware_store_makes_recall_edge_visible_on_append(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = setup_connection()?;
        let store = GraphAwareEventStore::new(conn.clone());
        let session_id = SessionId::new();
        let memory_id = MemoryId::new();

        let envelope = EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            EventKind::MemoryPinned,
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: "live graph recall hit".to_string(),
            session_id: Some(session_id),
            project_id: Some(ProjectId::new()),
            tx_id: None,
            rank: Some(1),
            source_tag: Some("recall".to_string()),
            query_text: Some("live graph".to_string()),
        }))?;

        store.append_event(&envelope)?;

        let vault = GraphVault::new(conn);
        let search = GraphSearch::new(&vault);
        assert_eq!(
            search.get_session_memories(&session_id.to_string())?,
            vec![memory_id.to_string()]
        );
        Ok(())
    }
}
