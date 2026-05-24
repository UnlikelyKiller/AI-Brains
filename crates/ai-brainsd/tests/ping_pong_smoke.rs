use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};

#[tokio::test]
#[allow(clippy::disallowed_methods)]
async fn test_daemon_ping_pong() -> Result<(), Box<dyn std::error::Error>> {
    let _temp_dir = tempfile::tempdir()?;

    // For now, let's just verify the DaemonRequest/Response serialization
    let ping = DaemonRequest::Ping;
    let json = serde_json::to_string(&ping).expect("serialize ping");
    let decoded: DaemonRequest = serde_json::from_str(&json).expect("deserialize ping");
    assert!(matches!(decoded, DaemonRequest::Ping));

    let pong = DaemonResponse::Pong;
    let json = serde_json::to_string(&pong).expect("serialize pong");
    let decoded: DaemonResponse = serde_json::from_str(&json).expect("deserialize pong");
    assert!(matches!(decoded, DaemonResponse::Pong));

    Ok(())
}
