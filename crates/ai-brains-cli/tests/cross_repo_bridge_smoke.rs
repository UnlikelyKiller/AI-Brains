use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use tempfile::tempdir;

#[test]
#[allow(clippy::disallowed_methods)]
fn test_cross_repo_sync_pull_and_push() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    // 1. Initialize the vault
    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2. Write a mock bridge export file (ChangeGuard -> AI-Brains)
    let pull_file_path = dir.path().join("cg_export.ndjson");
    let mut pull_file = File::create(&pull_file_path)?;

    // Hotspot record
    let record1 = serde_json::json!({
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-05-19T12:00:00Z",
        "parent_hash": null,
        "project_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "00000000-0000-0000-0000-000000000002",
        "tx_id": null,
        "record_kind": "hotspot",
        "payload": {
            "path": "crates/ai-brains-cli/src/main.rs",
            "score": 2.5,
            "reason": "high complexity"
        },
        "privacy": "LocalOnly"
    });

    // Parse and serialize matching standard implementation for exact hash
    let parsed1: ai_brains_contracts::bridge::BridgeRecord =
        serde_json::from_str(&serde_json::to_string(&record1)?)?;
    let json_for_hash = serde_json::to_string(&parsed1)?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(json_for_hash.as_bytes());
    let h1 = hex::encode(hasher.finalize());

    // Ledger delta record
    let record2 = serde_json::json!({
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-05-19T12:05:00Z",
        "parent_hash": h1,
        "project_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "00000000-0000-0000-0000-000000000002",
        "tx_id": "tx-12345",
        "record_kind": "ledger_delta",
        "payload": {
            "tx_id": "tx-12345",
            "intent": "refactor preflight",
            "files_changed": 1
        },
        "privacy": "LocalOnly"
    });

    writeln!(pull_file, "{}", serde_json::to_string(&record1)?)?;
    writeln!(pull_file, "{}", serde_json::to_string(&record2)?)?;
    pull_file.flush()?;

    // 3. Pull the mock records into AI-Brains
    let mut pull_cmd = Command::cargo_bin("ai-brains")?;
    pull_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("sync")
        .arg("pull")
        .arg("--from-file")
        .arg(&pull_file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully synced 2 records."));

    // 4. Insert a mock memory into memory_projection so that sync push has something to push
    {
        let key_str =
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string();
        let key = ai_brains_crypto::SqlCipherKey::from_raw(key_str);
        let conn = ai_brains_store::connection::VaultConnection::open(vault_path.clone(), &key)?;
        let conn_lock = conn.lock()?;
        conn_lock.execute(
            "INSERT INTO memory_projection (memory_id, session_id, project_id, content, privacy, status, level, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                "mem-123",
                "00000000-0000-0000-0000-000000000002",
                "00000000-0000-0000-0000-000000000001",
                "This is a mock level-1 memory insight.",
                "\"LocalOnly\"",
                "pinned",
                1,
                "2026-05-19T12:00:00Z",
                "2026-05-19T12:00:00Z"
            ]
        )?;
    }

    // Run sync push. It might fail if changeguard binary isn't in PATH,
    // but the NDJSON export file should have been written to the temp dir.
    let mut push_cmd = Command::cargo_bin("ai-brains")?;
    let push_assert = push_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("sync")
        .arg("push")
        .env(
            "AI_BRAINS_PROJECT_ID",
            "00000000-0000-0000-0000-000000000001",
        )
        .env(
            "AI_BRAINS_SESSION_ID",
            "00000000-0000-0000-0000-000000000002",
        )
        .assert();

    let output = push_assert.get_output();
    println!("PUSH STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("PUSH STDERR:\n{}", String::from_utf8_lossy(&output.stderr));

    push_assert.success();

    // Verify the export file contains a valid BridgeRecord with required fields.
    // Specific IDs vary by environment (dotenv_override loads from project .env),
    // so we check structural correctness rather than exact ID values.
    let export_path = std::env::temp_dir().join("aibrains_export.ndjson");
    assert!(
        export_path.exists(),
        "Export path aibrains_export.ndjson should exist"
    );

    let file = File::open(&export_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    if let Some(line) = lines.next() {
        let line = line?;
        let parsed: serde_json::Value = serde_json::from_str(&line)?;
        assert_eq!(parsed["direction"], "outbound");
        assert_eq!(parsed["record_kind"], "insight");
        // project_id and session_id must be present, non-empty strings
        assert!(parsed["project_id"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(parsed["session_id"].as_str().is_some_and(|s| !s.is_empty()));
        assert_eq!(parsed["privacy"], "ProjectLocal");
        assert!(parsed["payload"]["content"].as_str().is_some());
    }

    // Clean up
    let _ = std::fs::remove_file(export_path);

    Ok(())
}

#[test]
#[allow(clippy::disallowed_methods)]
fn test_cross_repo_e2e_integration_with_changeguard() -> Result<(), Box<dyn std::error::Error>> {
    // Check if changeguard CLI is available
    if std::process::Command::new("changeguard")
        .arg("--version")
        .output()
        .is_err()
    {
        println!("Skipping E2E test: changeguard CLI not found in PATH.");
        return Ok(());
    }

    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");
    let ws_path = dir.path().to_path_buf();

    // 1. Initialize ChangeGuard in the temp workspace
    let mut cg_init = std::process::Command::new("changeguard");
    cg_init.arg("init").current_dir(&ws_path);
    let output = cg_init.output()?;
    assert!(output.status.success(), "changeguard init failed");

    // Create a dummy source file so that scan has something to index
    let dummy_rs = ws_path.join("src").join("main.rs");
    std::fs::create_dir_all(ws_path.join("src"))?;
    std::fs::write(&dummy_rs, "fn main() { println!(\"hello\"); }")?;

    // Create a git repo and commit the file so changeguard scan detects it
    let mut git_init = std::process::Command::new("git");
    git_init.arg("init").current_dir(&ws_path).output()?;
    let mut git_add = std::process::Command::new("git");
    git_add
        .arg("add")
        .arg("src/main.rs")
        .current_dir(&ws_path)
        .output()?;
    let mut git_commit = std::process::Command::new("git");
    git_commit
        .arg("-c")
        .arg("user.name=Test")
        .arg("-c")
        .arg("user.email=test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("initial commit")
        .current_dir(&ws_path)
        .output()?;

    // Run changeguard scan with impact analysis
    let mut cg_scan = std::process::Command::new("changeguard");
    cg_scan.arg("scan").arg("--impact").current_dir(&ws_path);
    let output = cg_scan.output()?;
    assert!(output.status.success(), "changeguard scan failed");

    // 2. Initialize AI-Brains vault
    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 3. Initialize context in the workspace (which auto-triggers sync pull)
    let mut context_cmd = Command::cargo_bin("ai-brains")?;
    let output = context_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .current_dir(&ws_path)
        .output()?;
    println!(
        "CONTEXT STDOUT:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "CONTEXT STDERR:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());

    // 4. Verify that data was successfully pulled into the SQLite database.
    let key_str = "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string();
    let key = ai_brains_crypto::SqlCipherKey::from_raw(key_str);
    let conn = ai_brains_store::connection::VaultConnection::open(vault_path.clone(), &key)?;
    let conn_lock = conn.lock()?;

    // Check last_inbound_hash was set
    let last_inbound_hash: Option<String> = conn_lock
        .query_row(
            "SELECT value FROM sync_state WHERE key = 'last_inbound_hash'",
            [],
            |r| r.get(0),
        )
        .ok();
    assert!(
        last_inbound_hash.is_some(),
        "last_inbound_hash should have been populated by auto-trigger pull"
    );

    // 5. Insert a mock memory into memory_projection to push back to ChangeGuard
    let project_id_str = std::fs::read_to_string(ws_path.join(".env"))?
        .lines()
        .find(|l| l.starts_with("AI_BRAINS_PROJECT_ID"))
        .and_then(|l| l.split('=').nth(1))
        .unwrap()
        .to_string();

    let session_id_str = std::fs::read_to_string(ws_path.join(".env"))?
        .lines()
        .find(|l| l.starts_with("AI_BRAINS_SESSION_ID"))
        .and_then(|l| l.split('=').nth(1))
        .unwrap()
        .to_string();

    conn_lock.execute(
        "INSERT INTO memory_projection (memory_id, session_id, project_id, content, privacy, status, level, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            "mem-456",
            session_id_str,
            project_id_str,
            "Important architectural insight from E2E testing.",
            "\"LocalOnly\"",
            "pinned",
            1,
            "2026-05-19T12:00:00Z",
            "2026-05-19T12:00:00Z"
        ]
    )?;
    // Drop the lock so next Command invocation doesn't block on database access
    drop(conn_lock);
    drop(conn);

    // 6. Run sync push to export the memory back to ChangeGuard
    let mut push_cmd = Command::cargo_bin("ai-brains")?;
    push_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("sync")
        .arg("push")
        .current_dir(&ws_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully pushed insights to ChangeGuard.",
        ));

    // 7. Verify that ChangeGuard's latest-impact.json contains our pushed memory insight
    let latest_impact_path = ws_path
        .join(".changeguard")
        .join("reports")
        .join("latest-impact.json");
    assert!(
        latest_impact_path.exists(),
        "latest-impact.json should exist in .changeguard/reports/"
    );

    let impact_content = std::fs::read_to_string(&latest_impact_path)?;
    assert!(
        impact_content.contains("Important architectural insight from E2E testing."),
        "latest-impact.json should contain the exported memory insight content"
    );

    Ok(())
}

#[test]
#[allow(clippy::disallowed_methods)]
fn test_lineage_bootstrapping_with_existing_hash() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    // 1. Initialize the vault
    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2. Preset a mock last_inbound_hash in the sync_state table to simulate prior sync state
    {
        let key_str =
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string();
        let key = ai_brains_crypto::SqlCipherKey::from_raw(key_str);
        let conn = ai_brains_store::connection::VaultConnection::open(vault_path.clone(), &key)?;
        let conn_lock = conn.lock()?;
        conn_lock.execute(
            "INSERT INTO sync_state (key, value) VALUES (?, ?)",
            rusqlite::params!["last_inbound_hash", "some_prior_hash"],
        )?;
    }

    // 3. Write a mock bridge export file (ChangeGuard -> AI-Brains)
    let pull_file_path = dir.path().join("cg_export_bootstrap.ndjson");
    let mut pull_file = File::create(&pull_file_path)?;

    // Record 1: parent_hash: null (first record, bootstrapped)
    let record1 = serde_json::json!({
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-05-19T12:00:00Z",
        "parent_hash": null,
        "project_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "00000000-0000-0000-0000-000000000002",
        "tx_id": null,
        "record_kind": "hotspot",
        "payload": {
            "path": "crates/ai-brains-cli/src/main.rs",
            "score": 2.5,
            "reason": "high complexity"
        },
        "privacy": "LocalOnly"
    });

    // Parse and serialize matching standard implementation for exact hash
    let parsed1: ai_brains_contracts::bridge::BridgeRecord =
        serde_json::from_str(&serde_json::to_string(&record1)?)?;
    let json_for_hash = serde_json::to_string(&parsed1)?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(json_for_hash.as_bytes());
    let h1 = hex::encode(hasher.finalize());

    // Record 2: matches record 1's hash (should be pulled)
    let record2 = serde_json::json!({
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-05-19T12:05:00Z",
        "parent_hash": h1,
        "project_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "00000000-0000-0000-0000-000000000002",
        "tx_id": "tx-12345",
        "record_kind": "ledger_delta",
        "payload": {
            "tx_id": "tx-12345",
            "intent": "refactor preflight",
            "files_changed": 1
        },
        "privacy": "LocalOnly"
    });

    // Record 3: mismatching parent_hash (should be rejected/skipped)
    let record3 = serde_json::json!({
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-05-19T12:10:00Z",
        "parent_hash": "mismatching_parent_hash",
        "project_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "00000000-0000-0000-0000-000000000002",
        "tx_id": null,
        "record_kind": "hotspot",
        "payload": {
            "path": "crates/ai-brains-cli/src/main.rs",
            "score": 1.0,
            "reason": "low complexity"
        },
        "privacy": "LocalOnly"
    });

    writeln!(pull_file, "{}", serde_json::to_string(&record1)?)?;
    writeln!(pull_file, "{}", serde_json::to_string(&record2)?)?;
    writeln!(pull_file, "{}", serde_json::to_string(&record3)?)?;
    pull_file.flush()?;

    // 4. Pull the mock records into AI-Brains. Only record 1 & 2 should succeed.
    let mut pull_cmd = Command::cargo_bin("ai-brains")?;
    pull_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("sync")
        .arg("pull")
        .arg("--from-file")
        .arg(&pull_file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully synced 2 records."));

    Ok(())
}
