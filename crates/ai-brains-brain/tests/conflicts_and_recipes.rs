#![allow(clippy::disallowed_methods)]

use ai_brains_brain::NightlyService;
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    constructors::EventBuilder, Actor, AggregateType, EventKind, MemoryPinnedPayload, Payload,
    ProjectRegisteredPayload, SessionCompletedPayload, SessionStartedPayload,
    UserPromptRecordedPayload,
};
use ai_brains_models::{CompletionResponse, MockProvider};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_conflict_and_recipe_detection() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store = vault.clone();

    // 1. Setup existing state
    let project_id = ProjectId::new();
    let actor = Actor::System;

    // Register project
    let reg_event = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        EventKind::ProjectRegistered,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "Test Project".to_string(),
    }))?;
    event_store.append_event(&reg_event)?;

    // Add a memory (e.g. "Use standard port 8080")
    let memory_id = MemoryId::new();
    let mem_event = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryPinned,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: "Always use port 8080 for the dev server.".to_string(),
        session_id: None,
    }))?;
    event_store.append_event(&mem_event)?;

    // 2. Start a session that contradicts it
    let session_id = SessionId::new();
    let start_event = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::SessionStarted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id,
        project_id,
    }))?;
    event_store.append_event(&start_event)?;

    let prompt_event = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::UserPromptRecorded,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id,
        content: "We decided to change the dev port to 9000 because 8080 is blocked by the new security policy. Also, here is the workaround: 1. Stop service. 2. Run port-fix.ps1. 3. Restart.".to_string(),
    }))?;
    event_store.append_event(&prompt_event)?;

    let comp_event = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::SessionCompleted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionCompleted(SessionCompletedPayload {
        session_id,
    }))?;
    event_store.append_event(&comp_event)?;

    // Verify search works
    let search_results = query_store.search_memories("port", 10)?;
    assert!(
        !search_results.is_empty(),
        "FTS should find the memory about port"
    );

    // 3. Mock Provider responses
    // Response 1: Summary
    // Response 2: Conflict Detection (Contradiction found)
    // Response 3: Recipe Promotion (Recipe found)
    let mock_provider = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: "User changed the dev port to 9000.".to_string(),
            model: "mock-model".to_string(),
        },
        CompletionResponse {
            text: "CONFLICT: The new port 9000 contradicts the existing memory that says always use 8080.".to_string(),
            model: "mock-model".to_string(),
        },
        CompletionResponse {
            text: "Name: Port Fix Workaround\n- Stop service\n- Run port-fix.ps1\n- Restart".to_string(),
            model: "mock-model".to_string(),
        },
    ]));

    // 4. Run Nightly Service
    let nightly = NightlyService::new(
        query_store,
        event_store.clone(),
        mock_provider.clone(),
        mock_provider,
    );
    let count = nightly.run_nightly(project_id).await?;
    assert_eq!(count, 1);

    // 5. Verify Projections
    let conn = vault.lock()?;

    // Verify Conflict
    let conflict_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM conflict_projection", [], |row| {
            row.get(0)
        })?;
    assert_eq!(conflict_count, 1);

    // Verify Recipe
    let recipe_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM recipe_projection", [], |row| {
            row.get(0)
        })?;
    assert_eq!(recipe_count, 1);

    let recipe_name: String =
        conn.query_row("SELECT name FROM recipe_projection LIMIT 1", [], |row| {
            row.get(0)
        })?;
    assert_eq!(recipe_name, "Port Fix Workaround");

    Ok(())
}
