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
