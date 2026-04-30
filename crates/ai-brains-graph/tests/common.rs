#![allow(dead_code)]

use ai_brains_core::ids::MemoryId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    constructors::EventBuilder,
    payload::{MemoryPinnedPayload, ProjectRegisteredPayload, SessionStartedPayload},
    Actor, AggregateType, EventKind, Payload,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use tempfile::NamedTempFile;

pub fn setup_store() -> Result<EventStore, Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file
        .path()
        .to_str()
        .ok_or("invalid temp path")?
        .to_string();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let mut conn = VaultConnection::open(&db_path, &sql_key)?;
    conn.migrate()?;
    Ok(EventStore::new(conn))
}

pub fn append_session(
    store: &mut EventStore,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let session_id = ai_brains_core::ids::SessionId::new();
    let project_id = ai_brains_core::ids::ProjectId::new();

    let project_payload = Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "test-project".to_string(),
    });
    let project_envelope = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        EventKind::ProjectRegistered,
        Actor::System,
        Privacy::CloudOk,
    )
    .build(project_payload)?;
    store.append(&project_envelope)?;

    let payload = Payload::SessionStarted(SessionStartedPayload {
        session_id,
        project_id,
    });
    let envelope = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::SessionStarted,
        Actor::System,
        Privacy::CloudOk,
    )
    .build(payload)?;
    store.append(&envelope)?;

    Ok((session_id.to_string(), project_id.to_string()))
}
