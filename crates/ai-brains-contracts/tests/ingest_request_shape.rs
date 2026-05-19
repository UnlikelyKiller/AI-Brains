#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use uuid::Uuid;

#[test]
fn test_ingest_request_shape() {
    let request = IngestRequest {
        session_id: SessionId::from_uuid(Uuid::nil()),
        project_id: ai_brains_core::ids::ProjectId::from_uuid(Uuid::nil()),
        harness_id: HarnessId::from_uuid(Uuid::nil()),
        turn_id: TurnId::from_uuid(Uuid::nil()),
        role: "user".to_string(),
        content: "hello world".to_string(),
        privacy: Privacy::CloudOk,
        thinking: None,
        tx_id: None,
    };

    let serialized =
        serde_json::to_string(&request).expect("Should serialize ingest request in test");
    let expected = r#"{"session_id":"00000000-0000-0000-0000-000000000000","project_id":"00000000-0000-0000-0000-000000000000","harness_id":"00000000-0000-0000-0000-000000000000","turn_id":"00000000-0000-0000-0000-000000000000","role":"user","content":"hello world","privacy":"CloudOk"}"#;
    assert_eq!(serialized, expected);
}
