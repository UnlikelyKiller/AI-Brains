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
async fn test_nightly_summarizes_session() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );

    let vault = Arc::new(VaultConnection::open(db_path, &key)?);
    vault.migrate()?;
    let event_store = Arc::new(SqliteEventStore::new(vault.as_ref().clone()));

    // 1. Setup a completed session
    let project_id = ProjectId::new();
    let session_id = SessionId::new();

    let events = vec![
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Project,
            project_id.as_uuid(),
            ai_brains_events::EventKind::ProjectRegistered,
            ai_brains_events::Actor::User(ai_brains_core::ids::UserId::new()),
            Default::default(),
        )
        .build(Payload::ProjectRegistered(
            ai_brains_events::ProjectRegisteredPayload {
                project_id,
                name: "Test Project".to_string(),
                tx_id: None,
            },
        ))?,
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionStarted,
            ai_brains_events::Actor::User(ai_brains_core::ids::UserId::new()),
            Default::default(),
        )
        .build(Payload::SessionStarted(SessionStartedPayload {
            session_id,
            project_id,
            tx_id: None,
        }))?,
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::UserPromptRecorded,
            ai_brains_events::Actor::User(ai_brains_core::ids::UserId::new()),
            Default::default(),
        )
        .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
            session_id,
            content: "Hello".to_string(),
            tx_id: None,
        }))?,
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionCompleted,
            ai_brains_events::Actor::User(ai_brains_core::ids::UserId::new()),
            Default::default(),
        )
        .build(Payload::SessionCompleted(SessionCompletedPayload {
            session_id,
        }))?,
    ];

    for event in events {
        event_store.append_event(&event)?;
    }

    // 2. Rebuild projections so nightly service can see it
    let mut store_for_replay = SqliteEventStore::new(vault.as_ref().clone());
    store_for_replay.rebuild_projections()?;

    // 3. Run nightly service with mock provider
    let mock_provider = Arc::new(MockProvider::new(vec![
        ai_brains_models::CompletionResponse {
            text: "Summary of the session.".to_string(),
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
    ]));

    let nightly = NightlyService::new(
        vault.clone(),
        event_store.clone(),
        mock_provider.clone(),
        mock_provider,
    );
    let summarized_count = nightly.run_nightly(ProjectId::new()).await?;

    assert_eq!(summarized_count, 1);

    // 4. Verify summary event was appended
    let all_events = event_store.read_all_events()?;
    let summary_event = all_events
        .iter()
        .find(|e| matches!(e.payload, Payload::SessionSummaryCreated(_)));
    assert!(summary_event.is_some());

    Ok(())
}
