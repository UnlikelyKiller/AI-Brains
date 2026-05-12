#![allow(clippy::disallowed_methods)]

use ai_brains_brain::AggregatedLearningsService;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    constructors::EventBuilder, Actor, AggregateType, EventKind, MemorySynthesizedPayload, Payload,
    ProjectRegisteredPayload,
};
use ai_brains_models::{CompletionResponse, MockProvider};
use ai_brains_store::{EventStore, SqliteEventStore, VaultConnection};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_cross_agent_synthesis_level_2() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);

    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store = vault.clone();

    // 1. Setup existing state: 2 Level 1 memories
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

    // Level 0 Memory A1
    let memory_0a1_id = MemoryId::new();
    let event_0a1 = EventBuilder::new(
        AggregateType::Memory,
        memory_0a1_id.as_uuid(),
        EventKind::MemorySynthesized,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
        memory_id: memory_0a1_id,
        content: "Level 0 Memory A1".to_string(),
        source_memory_ids: vec![],
        level: 0,
        project_id,
    }))?;
    event_store.append_event(&event_0a1)?;

    // Level 0 Memory A2
    let memory_0a2_id = MemoryId::new();
    let event_0a2 = EventBuilder::new(
        AggregateType::Memory,
        memory_0a2_id.as_uuid(),
        EventKind::MemorySynthesized,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
        memory_id: memory_0a2_id,
        content: "Level 0 Memory A2".to_string(),
        source_memory_ids: vec![],
        level: 0,
        project_id,
    }))?;
    event_store.append_event(&event_0a2)?;

    // Level 1 Memory A
    let memory_1a_id = MemoryId::new();
    let event_1a = EventBuilder::new(
        AggregateType::Memory,
        memory_1a_id.as_uuid(),
        EventKind::MemorySynthesized,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
        memory_id: memory_1a_id,
        content: "Synthesis A: Focus on auth layer security.".to_string(),
        source_memory_ids: vec![memory_0a1_id, memory_0a2_id],
        level: 1,
        project_id,
    }))?;
    event_store.append_event(&event_1a)?;

    // Level 0 Memory B1
    let memory_0b1_id = MemoryId::new();
    let event_0b1 = EventBuilder::new(
        AggregateType::Memory,
        memory_0b1_id.as_uuid(),
        EventKind::MemorySynthesized,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
        memory_id: memory_0b1_id,
        content: "Level 0 Memory B1".to_string(),
        source_memory_ids: vec![],
        level: 0,
        project_id,
    }))?;
    event_store.append_event(&event_0b1)?;

    // Level 0 Memory B2
    let memory_0b2_id = MemoryId::new();
    let event_0b2 = EventBuilder::new(
        AggregateType::Memory,
        memory_0b2_id.as_uuid(),
        EventKind::MemorySynthesized,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
        memory_id: memory_0b2_id,
        content: "Level 0 Memory B2".to_string(),
        source_memory_ids: vec![],
        level: 0,
        project_id,
    }))?;
    event_store.append_event(&event_0b2)?;

    // Level 1 Memory B
    let memory_1b_id = MemoryId::new();
    let event_1b = EventBuilder::new(
        AggregateType::Memory,
        memory_1b_id.as_uuid(),
        EventKind::MemorySynthesized,
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
        memory_id: memory_1b_id,
        content: "Synthesis B: Focus on auth layer performance.".to_string(),
        source_memory_ids: vec![memory_0b1_id, memory_0b2_id],
        level: 1,
        project_id,
    }))?;
    event_store.append_event(&event_1b)?;

    // 2. Mock Provider for Level 2 Synthesis
    let mock_provider = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: "LEVEL 2: Global auth layer hardening and optimization.".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "SUPPORTED".to_string(),
            model: "mock".to_string(),
        },
    ]));

    // 3. Run Aggregated Learnings Service
    let cross_agent =
        AggregatedLearningsService::new(query_store, event_store.clone(), mock_provider);
    let count = cross_agent.run_cross_agent_synthesis(project_id).await?;
    assert_eq!(count, 1);

    // 4. Verify Projections
    let conn = vault.lock()?;

    // Verify Level 2 memory
    let level_2_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE level = 2",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(level_2_count, 1);

    // Verify content
    let content: String = conn.query_row(
        "SELECT content FROM memory_projection WHERE level = 2",
        [],
        |row| row.get(0),
    )?;
    assert!(content.contains("Global auth layer"));

    Ok(())
}
