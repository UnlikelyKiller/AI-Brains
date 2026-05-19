#![allow(clippy::disallowed_methods)]

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    constructors::EventBuilder, payload::MemoryPinnedPayload, Actor, AggregateType, EventKind,
    Payload,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brains_store::fts::search_memory;
use tempfile::NamedTempFile;

#[test]
fn test_fts_indexes_memory() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();

    let store = SqliteEventStore::new(conn);

    #[allow(unused_variables)]
    let project_id = ProjectId::new();
    let memory_id = MemoryId::new();
    let actor = Actor::System;

    let payload = Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: "The specific architectural nuance of event sourcing is immutability.".to_string(),
        session_id: None,
        project_id: None,
        tx_id: None,
    });

    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryPinned,
        actor,
        Privacy::LocalOnly,
    )
    .build(payload)
    .unwrap();

    store
        .append_event(&envelope)
        .expect("Failed to append event");

    // Search
    let conn_guard = store.connection().lock().unwrap();
    let results = search_memory(&conn_guard, "architectural nuance").expect("Search failed");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory_id, memory_id.to_string());
}
