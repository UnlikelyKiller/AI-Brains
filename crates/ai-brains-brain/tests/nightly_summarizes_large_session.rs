#![allow(clippy::disallowed_methods)]

use ai_brains_brain::NightlyService;
use ai_brains_core::ids::{ProjectId, SessionId};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::{
    Payload, SessionCompletedPayload, SessionStartedPayload, UserPromptRecordedPayload,
};
use ai_brains_models::mock::MockProvider;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_nightly_summarizes_large_session_via_chunking(
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt::try_init();
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );

    let vault = Arc::new(VaultConnection::open(db_path, &key)?);
    vault.migrate()?;
    let event_store = Arc::new(SqliteEventStore::new(vault.as_ref().clone()));

    // 1. Set environment variable for small context to force chunking
    // Overhead buffer is 1500.
    // effective_budget = 1600 - 1500 = 100.
    std::env::set_var("AI_BRAINS_CTX_SIZE", "1600");

    // 2. Setup a session with 2 turns to exceed the 100 token budget
    let project_id = ProjectId::new();
    let session_id = SessionId::new();

    let mut events = vec![
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Project,
            project_id.as_uuid(),
            ai_brains_events::EventKind::ProjectRegistered,
            ai_brains_events::Actor::System,
            Default::default(),
        )
        .build(Payload::ProjectRegistered(
            ai_brains_events::ProjectRegisteredPayload {
                project_id,
                name: "Test Project".to_string(),
            },
        ))?,
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionStarted,
            ai_brains_events::Actor::System,
            Default::default(),
        )
        .build(Payload::SessionStarted(SessionStartedPayload {
            session_id,
            project_id,
        }))?,
    ];

    // Add 2 long turns (~60 words each -> ~60 tokens each)
    let turn_content = "This is a moderately long turn content that should be around sixty words long to ensure it contributes significantly to the token count calculation in our test environment so we can reliably trigger the sequential chunking logic that we just implemented for the nightly summarization pipeline correctly and verify the context carryover mechanism works.";
    for _ in 0..2 {
        events.push(
            ai_brains_events::constructors::EventBuilder::new(
                ai_brains_events::AggregateType::Session,
                session_id.as_uuid(),
                ai_brains_events::EventKind::UserPromptRecorded,
                ai_brains_events::Actor::System,
                Default::default(),
            )
            .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
                session_id,
                content: turn_content.to_string(),
            }))?,
        );
    }

    events.push(
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionCompleted,
            ai_brains_events::Actor::System,
            Default::default(),
        )
        .build(Payload::SessionCompleted(SessionCompletedPayload {
            session_id,
        }))?,
    );

    for event in events {
        event_store.append_event(&event)?;
    }

    // 3. Rebuild projections
    let mut store_for_replay = SqliteEventStore::new(vault.as_ref().clone());
    store_for_replay.rebuild_projections()?;

    // 4. Run nightly service with mock provider
    // Expect 2 chunks.
    let mock_provider = Arc::new(MockProvider::new(vec![
        ai_brains_models::CompletionResponse {
            text: "{\"title\": \"Part 1 Summary\"}".to_string(),
            model: "mock".to_string(),
        },
        ai_brains_models::CompletionResponse {
            text: "{\"title\": \"Final synthesized summary\"}".to_string(),
            model: "mock".to_string(),
        },
        ai_brains_models::CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "mock".to_string(),
        },
        ai_brains_models::CompletionResponse {
            text: "NO RECIPE".to_string(),
            model: "mock".to_string(),
        },
        ai_brains_models::CompletionResponse {
            text: "SYNTHESIS".to_string(),
            model: "mock".to_string(),
        },
    ]));

    let nightly = NightlyService::new(
        vault.clone(),
        event_store.clone(),
        mock_provider.clone(),
        mock_provider,
    );
    let summarized_count = nightly.run_nightly(ProjectId::new()).await?;

    assert_eq!(summarized_count, 1);

    // 5. Verify final summary event
    let all_events = event_store.read_all_events()?;
    let summary_event = all_events
        .iter()
        .find(|e| matches!(e.payload, Payload::SessionSummaryCreated(_)))
        .expect("Should have a summary event");

    if let Payload::SessionSummaryCreated(payload) = &summary_event.payload {
        println!("Actual summary in event: {}", payload.summary);
        assert!(payload.summary.contains("Final synthesized summary"));
    } else {
        panic!("Wrong payload type");
    }

    Ok(())
}
