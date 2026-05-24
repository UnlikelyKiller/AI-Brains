#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::{ApiError, ApiResult};
use serde_json::json;

#[test]
fn test_api_success_shape() {
    let result = ApiResult::success(json!({"foo": "bar"}));
    let serialized =
        serde_json::to_string(&result).expect("Should serialize success response in test");
    let expected = r#"{"ok":true,"status":"success","data":{"foo":"bar"}}"#;
    assert_eq!(serialized, expected);
}

#[test]
fn test_api_error_shape() {
    let error = ApiError::new("ERR001", "Something went wrong");
    let result: ApiResult<()> = ApiResult::error(error);
    let serialized =
        serde_json::to_string(&result).expect("Should serialize error response in test");
    let expected = r#"{"ok":false,"status":"error","message":"Something went wrong","error":{"code":"ERR001","message":"Something went wrong"}}"#;
    assert_eq!(serialized, expected);
}

#[test]
fn test_api_warnings_shape() {
    let result = ApiResult::success(json!({})).with_warnings(vec!["Warning 1".to_string()]);
    let serialized =
        serde_json::to_string(&result).expect("Should serialize warning response in test");
    let expected = r#"{"ok":true,"status":"success","data":{},"warnings":["Warning 1"]}"#;
    assert_eq!(serialized, expected);
}
