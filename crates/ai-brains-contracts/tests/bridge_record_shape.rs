use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_core::privacy::Privacy;

#[test]
fn test_bridge_record_serde() -> Result<(), Box<dyn std::error::Error>> {
    let project_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
    let session_id = Some("660e8400-e29b-41d4-a716-446655440001".to_string());
    let tx_id = Some("test-tx".to_string());

    #[allow(clippy::disallowed_methods)]
    let record = BridgeRecord {
        bridge_version: "1.0".to_string(),
        direction: BridgeDirection::Inbound,
        timestamp: "2026-05-19T00:00:00Z".to_string(),
        parent_hash: None,
        project_id: project_id.clone(),
        session_id: session_id.clone(),
        tx_id: tx_id.clone(),
        record_kind: "prompt".to_string(),
        payload: serde_json::json!({"text": "hello"}),
        privacy: Privacy::LocalOnly,
    };

    let serialized = serde_json::to_string(&record)?;
    let deserialized: BridgeRecord = serde_json::from_str(&serialized)?;

    assert_eq!(deserialized.bridge_version, "1.0");
    assert_eq!(deserialized.project_id, project_id);
    assert_eq!(deserialized.session_id, session_id);
    assert_eq!(deserialized.tx_id, tx_id);
    assert_eq!(deserialized.record_kind, "prompt");

    // Verify null session_id deserialization works (ChangeGuard sends null)
    let json_with_null = r#"{"bridge_version":"0.2","direction":"inbound","timestamp":"2026-05-19T00:00:00Z","parent_hash":null,"project_id":"ChangeGuard","session_id":null,"tx_id":null,"record_kind":"hotspot_delta","payload":{},"privacy":"ProjectLocal"}"#;
    let deserialized: BridgeRecord = serde_json::from_str(json_with_null)?;
    assert_eq!(deserialized.project_id, "ChangeGuard");
    assert_eq!(deserialized.session_id, None);
    Ok(())
}
