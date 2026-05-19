#![allow(clippy::disallowed_methods)]

use ai_brains_brain::RetentionService;
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, EventKind, MemoryForgottenPayload, Payload, SessionStartedPayload,
    UserPromptRecordedPayload,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brains_store::QueryStore;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_forget_excludes_from_search() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);

    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store = vault.clone();

    // 1. Create a memory
    let memory_id = MemoryId::new();
    let event = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryPinned,
        Actor::User(ai_brains_core::ids::UserId::new()),
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(
        ai_brains_events::MemoryPinnedPayload {
            memory_id,
            content: "Remember this secret recipe.".to_string(),
            session_id: None,
            project_id: None,
            tx_id: None,
        },
    ))?;
    event_store.append_event(&event)?;

    // 2. Verify it's searchable
    let results = query_store.search_memories("secret", 10)?;
    assert_eq!(results.len(), 1);

    // 3. Forget it
    let forget_event = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryForgotten,
        Actor::User(ai_brains_core::ids::UserId::new()),
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryForgotten(MemoryForgottenPayload {
        memory_id,
    }))?;
    event_store.append_event(&forget_event)?;

    // 4. Verify it's NO LONGER searchable
    let results = query_store.search_memories("secret", 10)?;
    assert_eq!(results.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_retention_removes_old_turns() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);

    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store = vault.clone();

    // 1. Create a session with a turn
    let project_id = ProjectId::new();
    let session_id = SessionId::new();
    let actor = Actor::System;

    // Register project (FK requirement)
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        EventKind::ProjectRegistered,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(
        ai_brains_events::ProjectRegisteredPayload {
            project_id,
            name: "Test".to_string(),
            tx_id: None,
        },
    ))?;
    event_store.append_event(&reg)?;

    let start = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::SessionStarted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id,
        project_id,
        tx_id: None,
    }))?;
    event_store.append_event(&start)?;

    let prompt = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::UserPromptRecorded,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id,
        content: "Old turn".to_string(),
        tx_id: None,
    }))?;
    event_store.append_event(&prompt)?;

    // 2. Verify turn exists
    let turns = query_store.get_session_turns(&session_id.to_string())?;
    assert_eq!(turns.len(), 1);

    // 3. Manually update last_accessed_at to 100 days ago
    {
        let conn = vault.lock()?;
        let old_date = Utc::now() - Duration::days(100);
        conn.execute(
            "UPDATE turn_projection SET last_accessed_at = ?",
            [old_date.to_rfc3339()],
        )?;
    }

    // 4. Run retention (90 days)
    let retention = RetentionService::new(query_store.clone(), 90);
    retention.run_cleanup().await?;

    // 5. Verify turn is GONE
    let turns = query_store.get_session_turns(&session_id.to_string())?;
    assert_eq!(turns.len(), 0);

    Ok(())
}
