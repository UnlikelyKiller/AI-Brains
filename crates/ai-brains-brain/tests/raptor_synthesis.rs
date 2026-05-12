#![allow(clippy::disallowed_methods)]

use ai_brains_brain::NightlyService;
use ai_brains_core::ids::{ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    constructors::EventBuilder, Actor, AggregateType, EventKind, Payload, ProjectRegisteredPayload,
    SessionCompletedPayload, SessionStartedPayload, UserPromptRecordedPayload,
};
use ai_brains_models::{CompletionResponse, MockProvider};
use ai_brains_store::{EventStore, SqliteEventStore, VaultConnection};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_raptor_synthesis() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);

    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store = vault.clone();

    // 1. Setup existing state: 2 sessions
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

    // Session 1
    let session_1_id = SessionId::new();
    let start_1 = EventBuilder::new(
        AggregateType::Session,
        session_1_id.as_uuid(),
        EventKind::SessionStarted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id: session_1_id,
        project_id,
    }))?;
    event_store.append_event(&start_1)?;

    let prompt_1 = EventBuilder::new(
        AggregateType::Session,
        session_1_id.as_uuid(),
        EventKind::UserPromptRecorded,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: session_1_id,
        content: "Fixed a bug in the auth layer by adding a retry loop.".to_string(),
    }))?;
    event_store.append_event(&prompt_1)?;

    let comp_1 = EventBuilder::new(
        AggregateType::Session,
        session_1_id.as_uuid(),
        EventKind::SessionCompleted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionCompleted(SessionCompletedPayload {
        session_id: session_1_id,
    }))?;
    event_store.append_event(&comp_1)?;

    // Session 2
    let session_2_id = SessionId::new();
    let start_2 = EventBuilder::new(
        AggregateType::Session,
        session_2_id.as_uuid(),
        EventKind::SessionStarted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id: session_2_id,
        project_id,
    }))?;
    event_store.append_event(&start_2)?;

    let prompt_2 = EventBuilder::new(
        AggregateType::Session,
        session_2_id.as_uuid(),
        EventKind::UserPromptRecorded,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: session_2_id,
        content: "Optimized the auth database queries to reduce latency.".to_string(),
    }))?;
    event_store.append_event(&prompt_2)?;

    let comp_2 = EventBuilder::new(
        AggregateType::Session,
        session_2_id.as_uuid(),
        EventKind::SessionCompleted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionCompleted(SessionCompletedPayload {
        session_id: session_2_id,
    }))?;
    event_store.append_event(&comp_2)?;

    // 2. Mock Provider responses
    // Summary 1, Conflict 1, Recipe 1, Summary 2, Conflict 2, Recipe 2, Synthesis
    let mock_provider = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: "Session 1 summary about auth retry.".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO RECIPE".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "Session 2 summary about auth optimization.".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO RECIPE".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "SYNTHESIS: Ongoing auth layer hardening and performance work.".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "SUPPORTED".to_string(),
            model: "mock".to_string(),
        },
    ]));

    // 3. Run Nightly Service
    let nightly = NightlyService::new(
        query_store,
        event_store.clone(),
        mock_provider.clone(),
        mock_provider,
    );
    let count = nightly.run_nightly(ProjectId::new()).await?;
    assert_eq!(count, 2);

    // 4. Verify Projections
    let conn = vault.lock()?;

    // Verify Level 0 memories. Turn projection indexes raw turns for lexical recall,
    // and nightly adds session summaries at the same level.
    let level_0_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE level = 0",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(level_0_count, 4);

    // Verify Level 1 memory (synthesis)
    let synthesis_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE level = 1",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(synthesis_count, 1);

    // Verify Hierarchy
    let hierarchy_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM memory_hierarchy", [], |row| {
            row.get(0)
        })?;
    assert_eq!(hierarchy_count, 4); // 1 parent, 4 level-0 children

    Ok(())
}

#[tokio::test]
async fn test_crag_rejects_unsupported_synthesis() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);

    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store = vault.clone();

    // Setup 2 sessions
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

    // Session 1
    let session_1_id = SessionId::new();
    let start_1 = EventBuilder::new(
        AggregateType::Session,
        session_1_id.as_uuid(),
        EventKind::SessionStarted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id: session_1_id,
        project_id,
    }))?;
    event_store.append_event(&start_1)?;

    let prompt_1 = EventBuilder::new(
        AggregateType::Session,
        session_1_id.as_uuid(),
        EventKind::UserPromptRecorded,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: session_1_id,
        content: "Working on auth.".to_string(),
    }))?;
    event_store.append_event(&prompt_1)?;

    let comp_1 = EventBuilder::new(
        AggregateType::Session,
        session_1_id.as_uuid(),
        EventKind::SessionCompleted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionCompleted(SessionCompletedPayload {
        session_id: session_1_id,
    }))?;
    event_store.append_event(&comp_1)?;

    // Session 2
    let session_2_id = SessionId::new();
    let start_2 = EventBuilder::new(
        AggregateType::Session,
        session_2_id.as_uuid(),
        EventKind::SessionStarted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id: session_2_id,
        project_id,
    }))?;
    event_store.append_event(&start_2)?;

    let prompt_2 = EventBuilder::new(
        AggregateType::Session,
        session_2_id.as_uuid(),
        EventKind::UserPromptRecorded,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: session_2_id,
        content: "Optimizing auth.".to_string(),
    }))?;
    event_store.append_event(&prompt_2)?;

    let comp_2 = EventBuilder::new(
        AggregateType::Session,
        session_2_id.as_uuid(),
        EventKind::SessionCompleted,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SessionCompleted(SessionCompletedPayload {
        session_id: session_2_id,
    }))?;
    event_store.append_event(&comp_2)?;

    // Mock Provider with UNSUPPORTED verification
    let mock_provider = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: "Summary 1".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO RECIPE".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "Summary 2".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO RECIPE".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "SYNTHESIS: Something totally unrelated.".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "UNSUPPORTED".to_string(),
            model: "mock".to_string(),
        },
    ]));

    let nightly = NightlyService::new(
        query_store,
        event_store.clone(),
        mock_provider.clone(),
        mock_provider,
    );
    nightly.run_nightly(ProjectId::new()).await?;

    // Verify synthesis count is 0
    let conn = vault.lock()?;
    let synthesis_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE level = 1",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(synthesis_count, 0);

    Ok(())
}
