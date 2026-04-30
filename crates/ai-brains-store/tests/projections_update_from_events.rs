#![allow(clippy::disallowed_methods)]

use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    constructors::EventBuilder, payload::ProjectRegisteredPayload, Actor, AggregateType, EventKind,
    Payload,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;

#[test]
fn test_projections_update_from_events() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();

    let store = SqliteEventStore::new(conn);

    #[allow(unused_variables)]
    let project_id = ProjectId::new();
    let actor = Actor::System;

    let payload = Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "test".to_string(),
    });

    let envelope = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        EventKind::ProjectRegistered,
        actor,
        Privacy::LocalOnly,
    )
    .build(payload)
    .unwrap();

    store
        .append_event(&envelope)
        .expect("Failed to append event");

    // Verify projection
    let name: String = store
        .connection()
        .lock()
        .unwrap()
        .query_row(
            "SELECT name FROM project_projection WHERE project_id = ?",
            [project_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(name, "test");
}
