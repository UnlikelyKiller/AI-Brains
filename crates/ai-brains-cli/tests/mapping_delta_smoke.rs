use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
#[allow(clippy::disallowed_methods)]
fn test_project_mapping_and_delta_sync() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let vault_path = temp_dir.path().join("vault.db");

    // 1. Init vault
    let mut cmd = Command::cargo_bin("ai-brains")?;
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2. Setup project context
    let project_id = "00000000-0000-0000-0000-000000001234";
    let mut cmd = Command::cargo_bin("ai-brains")?;
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("context")
        .assert()
        .success();

    // 3. Create a mock agy transcript
    let agy_dir = temp_dir.path().join("agy-chats");
    std::fs::create_dir_all(&agy_dir)?;
    let transcript_path = agy_dir.join("transcript.jsonl");
    let mut file = std::fs::File::create(&transcript_path)?;
    writeln!(
        file,
        r#"{{"role": "user", "content": "hello", "timestamp": "2026-05-24T12:00:00Z"}}"#
    )?;

    let session_id = uuid::Uuid::new_v4().to_string();
    let project_hash = "abc123hash";

    let payload = serde_json::json!({
        "transcriptPath": transcript_path.to_string_lossy(),
        "sessionId": session_id,
        "projectHash": project_hash
    });

    // 4. Run agy-hook (should auto-link)
    let mut cmd = Command::cargo_bin("ai-brains")?;
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("agy-hook")
        .arg("--payload")
        .arg(serde_json::to_string(&payload)?)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Auto-linked projectHash abc123hash",
        ));

    // 5. Verify turn ingested
    let mut cmd = Command::cargo_bin("ai-brains")?;
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));

    // 6. Add another turn and run agy-hook again (delta sync)
    writeln!(
        file,
        r#"{{"role": "assistant", "content": "world", "timestamp": "2026-05-24T12:01:00Z"}}"#
    )?;

    let mut cmd = Command::cargo_bin("ai-brains")?;
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .arg("agy-hook")
        .arg("--payload")
        .arg(serde_json::to_string(&payload)?)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully ingested 1 turns")); // Should only ingest the new turn

    Ok(())
}
