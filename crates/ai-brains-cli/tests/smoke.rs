#![allow(clippy::disallowed_methods)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_cli_init_smoke() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    let mut cmd = Command::cargo_bin("ai-brains").unwrap();
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized successfully"));

    assert!(vault_path.exists());
}

#[test]
fn test_cli_ingest_smoke() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // Init first
    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Ingest
    let mut ingest_cmd = Command::cargo_bin("ai-brains").unwrap();
    let turn_json = r#"{
        "type": "turn",
        "session_id": "11111111-1111-1111-1111-111111111111",
        "project_id": "22222222-2222-2222-2222-222222222222",
        "harness_id": "33333333-3333-3333-3333-333333333333",
        "turn_id": "44444444-4444-4444-4444-444444444444",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "The password for the server is 'antigravity'."
    }"#;

    ingest_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"processed\":true"));
}

#[test]
fn test_cli_context_idempotency() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let env_path = dir.path().join(".env");

    // Init vault first (required for context)
    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // First run - initializes context
    let mut cmd1 = Command::cargo_bin("ai-brains").unwrap();
    cmd1.current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Context initialized for project"));

    assert!(env_path.exists());
    let content1 = std::fs::read_to_string(&env_path).unwrap();

    // Second run - should be idempotent and succeed without error
    let mut cmd2 = Command::cargo_bin("ai-brains").unwrap();
    cmd2.current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Context is already initialized for project",
        ));

    let content2 = std::fs::read_to_string(&env_path).unwrap();
    assert_eq!(
        content1, content2,
        "Context file should not have changed on second run"
    );

    // Third run with --new-session - should replace session and change file contents
    let mut cmd3 = Command::cargo_bin("ai-brains").unwrap();
    cmd3.current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .arg("--new-session")
        .assert()
        .success()
        .stdout(predicate::str::contains("Replacing existing session"));

    let content3 = std::fs::read_to_string(&env_path).unwrap();
    assert_ne!(
        content1, content3,
        "Context file should have changed after --new-session"
    );
}
