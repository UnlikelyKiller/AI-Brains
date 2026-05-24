#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::preflight::PreflightResponse;
use ai_brains_contracts::ApiResult;

#[test]
fn test_preflight_response_shape() {
    let response = PreflightResponse {
        daemon_version: "0.1.0".to_string(),
        vault_locked: false,
        system_healthy: true,
        capabilities: vec!["ingest".to_string(), "recall".to_string()],
    };
    let result = ApiResult::success(response);

    let serialized =
        serde_json::to_string(&result).expect("Should serialize preflight response in test");
    let expected = r#"{"ok":true,"status":"success","data":{"daemon_version":"0.1.0","vault_locked":false,"system_healthy":true,"capabilities":["ingest","recall"]}}"#;
    assert_eq!(serialized, expected);
}
