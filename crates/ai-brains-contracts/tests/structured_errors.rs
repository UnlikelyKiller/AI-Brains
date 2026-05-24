use ai_brains_contracts::response::{ApiError, ApiResult};

#[test]
#[allow(clippy::disallowed_methods)]
fn test_structured_error_serialization() {
    let error = ApiError::new("TEST_CODE", "test message");
    let result: ApiResult<serde_json::Value> = ApiResult::error(error);

    let json = serde_json::to_string(&result).expect("serialize failed");
    // Verify top-level fields for ChangeGuard compatibility
    assert!(json.contains(r#""status":"error""#));
    assert!(json.contains(r#""message":"test message""#));

    let decoded: ApiResult<serde_json::Value> =
        serde_json::from_str(&json).expect("deserialize failed");

    assert!(!decoded.ok);
    assert_eq!(decoded.status, "error");
    assert_eq!(decoded.message, Some("test message".to_string()));
    let error = decoded.error.expect("missing error object");
    assert_eq!(error.code, "TEST_CODE");
    assert_eq!(error.message, "test message");
}
