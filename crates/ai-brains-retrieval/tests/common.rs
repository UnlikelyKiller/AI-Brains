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
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;

pub fn store_with_memory(
    content: &str,
    privacy: Privacy,
) -> Result<SqliteEventStore, Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file
        .path()
        .to_str()
        .ok_or("invalid temp path")?
        .to_string();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(&db_path, &sql_key)?;
    conn.migrate()?;
    let store = SqliteEventStore::new(conn);

    let memory_id = MemoryId::new();
    let payload = Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.to_string(),
    });
    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryPinned,
        Actor::System,
        privacy,
    )
    .build(payload)?;
    store.append_event(&envelope)?;
    Ok(store)
}

pub fn append_active_session(
    store: &SqliteEventStore,
) -> Result<String, Box<dyn std::error::Error>> {
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
    store.append_event(&project_envelope)?;

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
    store.append_event(&envelope)?;
    Ok(session_id.to_string())
}

pub fn append_turn(
    store: &SqliteEventStore,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::Uuid::parse_str(session_id)?;
    let session_id = ai_brains_core::ids::SessionId::from_uuid(uuid);
    let payload = if role == "user" {
        Payload::UserPromptRecorded(ai_brains_events::payload::UserPromptRecordedPayload {
            session_id,
            content: content.to_string(),
        })
    } else {
        Payload::AssistantFinalRecorded(ai_brains_events::payload::AssistantFinalRecordedPayload {
            session_id,
            content: content.to_string(),
        })
    };

    let kind = if role == "user" {
        EventKind::UserPromptRecorded
    } else {
        EventKind::AssistantFinalRecorded
    };

    let envelope = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        kind,
        Actor::System,
        Privacy::CloudOk,
    )
    .build(payload)?;
    store.append_event(&envelope)?;
    Ok(())
}
