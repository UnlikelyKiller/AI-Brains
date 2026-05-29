use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_store::EventStore;
use std::str::FromStr;
use std::sync::Arc;

pub async fn run(
    ctx: &AppContext,
    schedule: bool,
    unschedule: bool,
    start_time: String,
    status: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let task_name = "AI-Brains-Nightly";

    if status {
        let query_store = ctx.conn.clone() as Arc<dyn ai_brains_store::QueryStore>;
        let unsummarized = query_store.get_unsummarized_sessions()?;
        let last_run = query_store.get_last_nightly_run()?;
        let last_count = query_store
            .get_sync_state("last_nightly_count")?
            .unwrap_or_else(|| "0".to_string());

        println!("=== Nightly Status ===");
        match last_run {
            Some(ts) => println!("Last nightly run: {}", ts),
            None => println!("Last nightly run: never"),
        }
        println!("Unsummarized sessions remaining: {}", unsummarized.len());
        println!("Sessions summarized in last run: {}", last_count);
        println!("======================");
        return Ok(());
    }

    if unschedule {
        let output = std::process::Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .output()
            .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

        if output.status.success() {
            println!("Nightly task '{}' removed.", task_name);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to remove task: {}", stderr);
        }
        return Ok(());
    }

    if schedule {
        let exe_path = std::env::current_exe()?;
        let exe_str = exe_path.to_str().ok_or("Invalid executable path")?;

        let output = std::process::Command::new("schtasks")
            .args([
                "/create",
                "/tn",
                task_name,
                "/tr",
                &format!("'{}' nightly", exe_str),
                "/sc",
                "daily",
                "/st",
                &start_time,
                "/f",
            ])
            .output()
            .map_err(|e| {
                format!(
                    "Failed to execute schtasks: {}. Run in an elevated PowerShell session.",
                    e
                )
            })?;

        if output.status.success() {
            println!(
                "Nightly task '{}' scheduled daily at {}.",
                task_name, start_time
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let cmd = ai_brains_scheduler::TaskScheduler::render_create_command(
                exe_str,
                task_name,
                &start_time,
            );
            eprintln!(
                "Failed to schedule task. Run this in an elevated PowerShell session:\n{}\nError: {}{}",
                cmd, stdout, stderr
            );
        }
        return Ok(());
    }

    // Ensure daemon is running for background intelligence sweep
    let daemon_client = crate::daemon_client::DaemonClient::new();
    if !daemon_client
        .ensure_running(&ctx.vault_path, &ctx._key)
        .await
    {
        tracing::warn!(
            "Failed to ensure daemon is running. Nightly sweep may have reduced functionality."
        );
    }

    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default();

    if project_id == ProjectId::default() {
        eprintln!(
            "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first. Using default project."
        );
    }

    let event_store = Arc::new(ai_brains_store::SqliteEventStore::new((*ctx.conn).clone()));
    let query_store = ctx.conn.clone() as Arc<dyn ai_brains_store::QueryStore>;

    let model_url = std::env::var("AI_BRAINS_MODEL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
    let completion_model = std::env::var("AI_BRAINS_COMPLETION_MODEL")
        .unwrap_or_else(|_| "gemma-4-E4B-it-Q6_K.gguf".to_string());

    let embedding_url = std::env::var("AI_BRAINS_EMBEDDING_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8083".to_string());
    let embedding_model = std::env::var("AI_BRAINS_EMBEDDING_MODEL")
        .unwrap_or_else(|_| "nomic-embed-text-v1.5".to_string());

    let completion_provider = Arc::new(ai_brains_models::llama_cpp::LlamaCppProvider::new(
        model_url,
        completion_model,
    ));
    let embedding_provider = Arc::new(ai_brains_models::llama_cpp::LlamaCppProvider::new(
        embedding_url,
        embedding_model,
    ));

    // Import Antigravity sessions before summarization so they get summarized too
    if let Err(e) = crate::commands::antigravity_import::run(ctx, 30) {
        tracing::error!("Antigravity import failed: {}", e);
    }

    let service = ai_brains_brain::NightlyService::new(
        query_store,
        event_store,
        completion_provider,
        embedding_provider,
    );

    eprintln!("Starting nightly intelligence sweep...");
    eprintln!("Summarizing sessions...");

    let count = service.run_nightly(project_id).await?;
    eprintln!("Running memory synthesis...");
    eprintln!("Nightly sweep completed. {} sessions summarized.", count);

    // --- MADR Ingestion (Phase 18: T41) ---
    eprintln!("Ingesting structured MADR decisions from ChangeGuard...");
    if let Err(e) = ingest_madr_from_changeguard(ctx, project_id) {
        tracing::error!("MADR ingestion failed (non-fatal): {}", e);
        eprintln!(
            "Note: MADR ingestion failed: {}. Nightly sweep completed successfully.",
            e
        );
    }

    Ok(())
}

/// Fetch structured MADR records from ChangeGuard via bridge IPC and ingest as
/// Decision domain events into the event store.
fn ingest_madr_from_changeguard(
    ctx: &AppContext,
    project_id: ProjectId,
) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_contracts::bridge::BridgeRecord;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let temp_path = {
        let mut p = std::env::temp_dir();
        p.push("cg_madr_export.ndjson");
        p
    };

    // Call ChangeGuard bridge export --ledger to fetch MADR records
    let output = std::process::Command::new("changeguard")
        .args([
            "bridge",
            "export",
            "--out",
            temp_path.to_str().ok_or("Invalid temp path")?,
            "--ledger",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!("ChangeGuard bridge export failed: {}", stderr);
            return Ok(()); // Non-fatal: fail gracefully
        }
        Err(e) => {
            tracing::warn!("ChangeGuard CLI not available: {}", e);
            return Ok(()); // Non-fatal: fail gracefully
        }
    }

    // Parse exported records looking for MADR entries
    let file = match File::open(&temp_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("Failed to open MADR export file: {}", e);
            return Ok(());
        }
    };
    let reader = BufReader::new(file);

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut ingested = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("Failed to read MADR export line: {}", e);
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to parse BridgeRecord in MADR export: {}", e);
                continue;
            }
        };

        // Only process MADR/decision records
        let record_kind_lower = record.record_kind.to_lowercase();
        if record_kind_lower != "madr" && record_kind_lower != "decision" {
            continue;
        }

        // Extract structured MADR fields from payload
        let payload = record.payload_value();
        let title = payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Decision")
            .to_string();
        let context = payload
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let decision = payload
            .get("decision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let consequences = payload
            .get("consequences")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if decision.is_empty() && context.is_empty() {
            continue; // Skip records without meaningful MADR content
        }

        // Parse record-level IDs
        let record_project_id = if !record.project_id.is_empty() {
            ProjectId::from_str(&record.project_id).ok()
        } else {
            None
        };
        let record_session_id = record
            .session_id
            .as_ref()
            .and_then(|s| ai_brains_core::ids::SessionId::from_str(s).ok());
        let tx_id = record
            .tx_id
            .as_ref()
            .map(|s| ai_brains_core::ids::TransactionId::new(s.clone()));

        // Build DecisionRecorded event
        let decision_id = MemoryId::new();
        let event = ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Decision,
            decision_id.as_uuid(),
            ai_brains_events::EventKind::DecisionRecorded,
            ai_brains_events::Actor::System,
            record.privacy,
        )
        .build(ai_brains_events::Payload::DecisionRecorded(
            ai_brains_events::DecisionRecordedPayload {
                decision_id,
                title,
                context,
                decision,
                consequences,
                project_id: record_project_id.or(Some(project_id)),
                session_id: record_session_id,
                tx_id,
            },
        ))?;

        event_store.append_event(&event)?;
        ingested += 1;
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    eprintln!("MADR ingestion completed. {} decisions ingested.", ingested);
    Ok(())
}

/// Format structured MADR fields into MADR-compliant markdown.
/// This is used by the projection handler; exposed here for testability.
#[allow(dead_code)]
pub fn format_madr_markdown(
    title: &str,
    context: &str,
    decision: &str,
    consequences: &str,
) -> String {
    format!(
        "# {}\n\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}",
        title, context, decision, consequences
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_madr_markdown_produces_expected_structure() {
        let result = format_madr_markdown(
            "ADR: Use SQLite",
            "We needed an embedded database.",
            "We chose SQLite with SQLCipher.",
            "Simpler deployment, encrypted at rest.",
        );

        assert!(result.contains("# ADR: Use SQLite"));
        assert!(result.contains("## Context"));
        assert!(result.contains("We needed an embedded database."));
        assert!(result.contains("## Decision"));
        assert!(result.contains("We chose SQLite with SQLCipher."));
        assert!(result.contains("## Consequences"));
        assert!(result.contains("Simpler deployment, encrypted at rest."));
    }

    #[test]
    fn format_madr_markdown_handles_empty_fields() {
        let result = format_madr_markdown("Title Only", "", "", "");

        assert!(result.contains("# Title Only"));
        assert!(result.contains("## Context\n\n"));
        assert!(result.contains("## Decision\n\n"));
        assert!(result.contains("## Consequences\n"));
    }
}
