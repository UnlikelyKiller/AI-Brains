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

/// T73: `init` on a vault that already contains ingested data must refuse
/// with a structured error unless `--force` is provided.
#[test]
fn test_init_refuses_populated_vault() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // First init + ingest to populate the vault with one project + one session.
    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "11111111-1111-1111-1111-111111111111",
        "project_id": "22222222-2222-2222-2222-222222222222",
        "harness_id": "33333333-3333-3333-3333-333333333333",
        "turn_id": "44444444-4444-4444-4444-444444444444",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Populate the vault so init has data to refuse on."
    }"#;

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // Second init without --force must fail with a clear error.
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "init on populated vault must exit non-zero; got: {:?}",
        output
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already") || stderr.contains("Refusing"),
        "stderr should explain the refusal; got: {stderr}"
    );
}

/// T73 companion: with `--force`, init must succeed even on a populated vault.
#[test]
fn test_init_force_overwrites() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "55555555-5555-5555-5555-555555555555",
        "project_id": "66666666-6666-6666-6666-666666666666",
        "harness_id": "77777777-7777-7777-7777-777777777777",
        "turn_id": "88888888-8888-8888-8888-888888888888",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Populate the vault so --force is exercised."
    }"#;

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized successfully"));
}

/// T74: After init + ingest + recall, the live graph projector should report
/// at least one node and one edge in `ai-brains graph update`. This catches
/// silent graph-projector regressions where ingest/recall succeed but no
/// graph state is written.
///
/// Gated on the `graph` feature because the `graph` subcommand is only
/// compiled in with that feature. Run with:
///   cargo nextest run -p ai-brains-cli --features graph test_graph_health_smoke
#[cfg(feature = "graph")]
#[test]
fn test_graph_health_smoke() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // 1) Init
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2) Ingest one turn so we have a project + session + turn.
    let turn_json = r#"{
        "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "project_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "harness_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "turn_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Anchoring memory for the graph health smoke test."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // 3) Pin a memory so the graph has a `MemoryPinned` event to project.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("pin")
        .arg("T74 graph health smoke seed")
        .assert()
        .success();

    // 4) Recall — T67 wiring emits MemoryPinned events for hits, which the
    //    live graph projector (T69) should immediately apply.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("graph health smoke")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // 5) `graph update` should report live, non-empty graph state.
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("graph")
        .arg("update")
        .output()
        .expect("graph update must run");

    assert!(
        output.status.success(),
        "graph update failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("graph update must emit valid JSON, got: {stdout} ({e})"));

    let nodes = parsed["nodes"].as_i64().unwrap_or(-1);
    let edges = parsed["edges"].as_i64().unwrap_or(-1);
    let status = parsed["status"].as_str().unwrap_or("");

    assert!(
        nodes >= 1,
        "graph must contain at least 1 node; got: {parsed}"
    );
    assert!(
        edges >= 1,
        "graph must contain at least 1 edge; got: {parsed}"
    );
    assert_eq!(status, "live", "graph status must be 'live'; got: {parsed}");
}

/// T76: `backup restore --dry-run` must verify integrity and report the plan,
/// but must not overwrite the destination vault and must not prompt.
#[test]
fn test_backup_restore_dry_run() {
    let dir = tempdir().unwrap();
    let source_vault = dir.path().join("source.db");
    let dest_vault = dir.path().join("dest.db");

    // Create source vault with a project so restore has real data to verify.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("init")
        .assert()
        .success();
    let turn_json = r#"{
        "session_id": "99999999-9999-9999-9999-999999999999",
        "project_id": "88888888-8888-8888-8888-888888888888",
        "harness_id": "77777777-7777-7777-7777-777777777777",
        "turn_id": "66666666-6666-6666-6666-666666666666",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Seed the source vault so backup has data."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // Create dest vault and seed it with a different project so we can detect
    // any accidental overwrite.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("init")
        .assert()
        .success();
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("pin")
        .arg("Original content on dest that must survive dry-run")
        .assert()
        .success();

    // Snapshot the dest vault size; dry-run must leave it untouched.
    let dest_size_before = std::fs::metadata(&dest_vault).unwrap().len();

    // Generate a backup of source.
    let backup_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("backup")
        .output()
        .expect("backup must run");
    assert!(backup_output.status.success());
    let stdout = String::from_utf8_lossy(&backup_output.stdout);
    // Output is "Backup created and verified: <path>"
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed")
        .trim();
    let backup_path = std::path::PathBuf::from(backup_path);
    assert!(backup_path.exists(), "backup file must exist");

    // Dry-run restore.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"));

    // The dest vault must be byte-for-byte untouched.
    let dest_size_after = std::fs::metadata(&dest_vault).unwrap().len();
    assert_eq!(
        dest_size_before, dest_size_after,
        "dry-run must not modify the destination vault"
    );
}

/// T76: `backup restore --force` must skip the interactive confirm prompt.
#[test]
fn test_backup_restore_force_skips_prompt() {
    let dir = tempdir().unwrap();
    let source_vault = dir.path().join("source.db");
    let dest_vault = dir.path().join("dest.db");

    // Build source vault with a project + a backup file.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("init")
        .assert()
        .success();
    let turn_json = r#"{
        "session_id": "44444444-4444-4444-4444-444444444444",
        "project_id": "33333333-3333-3333-3333-333333333333",
        "harness_id": "22222222-2222-2222-2222-222222222222",
        "turn_id": "11111111-1111-1111-1111-111111111111",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Source content for force-restore test."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let backup_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("backup")
        .output()
        .expect("backup must run");
    assert!(backup_output.status.success());
    let stdout = String::from_utf8_lossy(&backup_output.stdout);
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed")
        .trim();
    let backup_path = std::path::PathBuf::from(backup_path);

    // Init dest so the file exists (required for SQLite restore).
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("init")
        .assert()
        .success();

    // --force must succeed with no stdin (interactive prompt would hang).
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault restored from"));
}
