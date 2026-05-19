#![allow(clippy::disallowed_methods)]

use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, EventKind, Payload, ProjectRegisteredPayload, SessionStartedPayload,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brainsd::DaemonWriter;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unique_spool_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brainsd-{name}-{nanos}"))
}

fn request(project_id: ProjectId, session_id: SessionId, content: &str) -> IngestRequest {
    IngestRequest {
        session_id,
        project_id,
        harness_id: HarnessId::new(),
        turn_id: TurnId::new(),
        role: "user".to_string(),
        content: content.to_string(),
        privacy: Privacy::CloudOk,
        thinking: None,
        tx_id: None,
    }
}

#[tokio::test]
async fn spool_replays_after_restart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spool_dir = unique_spool_dir("replay");
    let db_path = spool_dir.join("vault.db");
    std::fs::create_dir_all(&spool_dir)?;

    let key = ai_brains_crypto::SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );
    let conn = VaultConnection::open(db_path, &key)?;
    conn.migrate()?;
    let store = Arc::new(SqliteEventStore::new(conn));

    let project_id = ProjectId::new();
    let session_id = SessionId::new();

    // Register project and session (FK requirements)
    let project_event = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        EventKind::ProjectRegistered,
        Actor::System,
        Privacy::CloudOk,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "test".to_string(),
        tx_id: None,
    }))?;
    store.append_event(&project_event)?;

    let session_event = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        EventKind::SessionStarted,
        Actor::System,
        Privacy::CloudOk,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id,
        project_id,
        tx_id: None,
    }))?;
    store.append_event(&session_event)?;

    // 1. Start writer, it should create the dir
    let _writer = DaemonWriter::start(spool_dir.clone(), store.clone()).await?;

    // 2. Add some items to spool manually (bypassing queue)
    let req1 = request(project_id, session_id, "manual one");
    let path1 = spool_dir.join("item1.json");
    let payload1 = serde_json::to_vec(&ai_brains_daemon_api::DaemonRequest::Ingest(req1))?;
    tokio::fs::write(&path1, payload1).await?;

    // 3. Restart writer (new instance on same dir)
    let writer2 = DaemonWriter::start(spool_dir.clone(), store).await?;

    // 4. Wait for background replay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 5. Verify items replayed
    let events = writer2.recorded_events().await;
    assert!(events.iter().any(|e| match &e.payload {
        ai_brains_events::Payload::UserPromptRecorded(p) => p.content == "manual one",
        _ => false,
    }));

    let _ = tokio::fs::remove_dir_all(spool_dir).await;
    Ok(())
}
