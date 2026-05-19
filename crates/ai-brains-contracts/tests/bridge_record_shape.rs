use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_core::ids::{ProjectId, SessionId, TransactionId};
use ai_brains_core::privacy::Privacy;

#[test]
fn test_bridge_record_serde() -> Result<(), Box<dyn std::error::Error>> {
    let project_id = ProjectId::new();
    let session_id = SessionId::new();
    let tx_id = TransactionId::new("test-tx".to_string());

    #[allow(clippy::disallowed_methods)]
    let record = BridgeRecord {
        bridge_version: "1.0".to_string(),
        direction: BridgeDirection::Inbound,
        timestamp: "2026-05-19T00:00:00Z".to_string(),
        parent_hash: None,
        project_id,
        session_id,
        tx_id: Some(tx_id),
        record_kind: "prompt".to_string(),
        payload: serde_json::json!({"text": "hello"}),
        privacy: Privacy::LocalOnly,
    };

    let serialized = serde_json::to_string(&record)?;
    let deserialized: BridgeRecord = serde_json::from_str(&serialized)?;

    assert_eq!(deserialized.bridge_version, "1.0");
    assert_eq!(deserialized.project_id, project_id);
    assert_eq!(deserialized.session_id, session_id);
    assert_eq!(
        deserialized.tx_id.ok_or("missing tx_id")?.as_str(),
        "test-tx"
    );
    assert_eq!(deserialized.record_kind, "prompt");
    Ok(())
}
