use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::TurnId;
use ai_brains_store::EventStore;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

struct TempFileCleanup {
    path: PathBuf,
}

impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn run_pull(
    ctx: &AppContext,
    from_file: Option<PathBuf>,
    hotspots: bool,
    ledger: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let _temp_cleanup;
    let actual_file = match from_file {
        Some(path) => {
            if !path.exists() {
                return Err(format!("File not found: {}", path.display()).into());
            }
            _temp_cleanup = None;
            path
        }
        None => {
            let temp_path = {
                let mut p = std::env::temp_dir();
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("Clock error: {}", e))?
                    .as_nanos();
                p.push(format!("cg_export_{}.ndjson", now));
                p
            };
            _temp_cleanup = Some(TempFileCleanup {
                path: temp_path.clone(),
            });

            let mut cmd = std::process::Command::new("changeguard");
            cmd.arg("bridge").arg("export");
            cmd.arg("--out").arg(&temp_path);

            let pull_hotspots = hotspots || !ledger;
            let pull_ledger = ledger || !hotspots;

            if pull_hotspots {
                cmd.arg("--hotspots");
            }
            if pull_ledger {
                cmd.arg("--ledger");
            }

            let output = cmd.output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to export from ChangeGuard: {}", stderr).into());
            }
            temp_path
        }
    };

    let file = File::open(&actual_file)?;
    let reader = BufReader::new(file);

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    let mut count = 0;
    let mut last_hash: Option<String> = sink.store.get_sync_state("last_inbound_hash")?;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str::<BridgeRecord>(&line) {
            Ok(r) => {
                if let Some(expected_last) = &last_hash {
                    match &r.parent_hash {
                        Some(actual_parent) => {
                            if actual_parent != expected_last {
                                eprintln!("Lineage verification failed: parent_hash mismatch. Expected {}, got {}. Skipping record.", expected_last, actual_parent);
                                continue;
                            }
                        }
                        None => {
                            eprintln!(
                                "Bridge record rejected: missing parent_hash. Expected {}",
                                expected_last
                            );
                            continue;
                        }
                    }
                }
                r
            }
            Err(err) => {
                eprintln!("Failed to parse BridgeRecord: {}. Skipping line.", err);
                continue;
            }
        };

        // Compute current record hash for the next iteration
        use sha2::{Digest, Sha256};
        let json_for_hash = serde_json::to_string(&record).unwrap_or_default();
        let mut hasher2 = Sha256::new();
        hasher2.update(json_for_hash.as_bytes());
        let hash_hex = format!("{:x}", hasher2.finalize());
        last_hash = Some(hash_hex.clone());
        sink.store.set_sync_state("last_inbound_hash", &hash_hex)?;

        // Pulling only inbound records (ChangeGuard -> AI-Brains)
        if record.direction != ai_brains_contracts::bridge::BridgeDirection::Inbound {
            continue;
        }

        // Parse string IDs from the interchange format into typed IDs.
        let project_id = ai_brains_core::ids::ProjectId::from_str(&record.project_id)
            .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new());
        let session_id = match &record.session_id {
            Some(s) => ai_brains_core::ids::SessionId::from_str(s)
                .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new()),
            None => ai_brains_core::ids::SessionId::new(),
        };
        let tx_id = record.tx_id.as_ref().map(|s| ai_brains_core::ids::TransactionId::new(s.clone()));

        // Apply Privacy::combine() during sync ingestion — combine incoming record privacy with project session privacy.
        let session_privacy = sink
            .store
            .get_session_privacy(&session_id.to_string())?
            .unwrap_or(record.privacy);
        let combined_privacy = record.privacy.combine(session_privacy);

        // Ensure context exists
        ctx.ensure_project_and_session_exists(
            &mut sink,
            &service,
            &capture_context,
            project_id,
            session_id,
            ai_brains_core::ids::HarnessId::default(), // Unknown harness
            combined_privacy,
        )?;

        // Map record to IngestRequest
        let role = match record.record_kind.to_lowercase().as_str() {
            "user" | "prompt" => "user",
            "assistant" | "response" | "final" => "assistant",
            _ => "assistant", // Default to assistant for external signals
        };

        let content = record.formatted_payload();

        // Handle specific structured payloads
        if record.record_kind == "verify_outcome" {
            if let Ok(outcome) = serde_json::from_value::<
                ai_brains_events::VerifyOutcomeRecordedPayload,
            >(record.payload.clone())
            {
                let event = ai_brains_events::constructors::EventBuilder::new(
                    ai_brains_events::AggregateType::System,
                    uuid::Uuid::new_v4(),
                    ai_brains_events::EventKind::VerifyOutcomeRecorded,
                    ai_brains_events::Actor::System,
                    combined_privacy,
                )
                .build(ai_brains_events::Payload::VerifyOutcomeRecorded(outcome))?;
                sink.store.append_event(&event)?;
                count += 1;
                continue;
            }
        }

        let request = IngestRequest {
            session_id,
            project_id,
            harness_id: ai_brains_core::ids::HarnessId::default(),
            turn_id: TurnId::new(),
            role: role.to_string(),
            content,
            thinking: None,
            privacy: combined_privacy,
            tx_id,
        };

        service.ingest_request(request, capture_context.clone(), &mut sink)?;
        count += 1;
    }

    println!("Successfully synced {} records.", count);
    Ok(())
}

#[allow(clippy::disallowed_methods, clippy::type_complexity)]
pub fn run_push(
    ctx: &AppContext,
    _with_impact: bool,
    _with_verify: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("AI-Brains: Exporting insights to ChangeGuard...");

    use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
    use std::io::Write;

    use std::str::FromStr;
    let project_id = if let Ok(val) = std::env::var("AI_BRAINS_PROJECT_ID") {
        ai_brains_core::ids::ProjectId::from_str(&val)?
    } else {
        ai_brains_core::ids::ProjectId::new()
    };
    let session_id = if let Ok(val) = std::env::var("AI_BRAINS_SESSION_ID") {
        ai_brains_core::ids::SessionId::from_str(&val)?
    } else {
        ai_brains_core::ids::SessionId::new()
    };

    let mut out_records = Vec::new();
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut last_hash: Option<String> = event_store.get_sync_state("last_outbound_hash")?;

    let rows_data: Vec<(String, String, String, Option<String>, Option<String>)> = {
        let conn = ctx.conn.lock()?;
        let mut stmt = conn.prepare("SELECT memory_id, content, privacy, project_id, session_id FROM memory_projection WHERE level > 0")?;
        let mut rows = stmt.query([])?;
        let mut data = Vec::new();
        while let Some(row) = rows.next()? {
            data.push((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ));
        }
        data
    };

    for (memory_id, content, privacy_json, row_project_id_str, row_session_id_str) in rows_data {
        let privacy = serde_json::from_str::<ai_brains_core::privacy::Privacy>(&privacy_json)
            .unwrap_or_default();
        if privacy == ai_brains_core::privacy::Privacy::NeverInject
            || privacy == ai_brains_core::privacy::Privacy::Sealed
        {
            continue;
        }

        let record_project_id = if let Some(pid_str) = row_project_id_str {
            ai_brains_core::ids::ProjectId::from_str(&pid_str).unwrap_or(project_id)
        } else {
            project_id
        };

        let record_session_id = if let Some(sid_str) = row_session_id_str {
            ai_brains_core::ids::SessionId::from_str(&sid_str).unwrap_or(session_id)
        } else {
            session_id
        };

        let payload = serde_json::json!({
            "type": "Insight",
            "memory_id": memory_id,
            "relevance": 1.0,
            "content": content
        });
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| String::new());
        let record = BridgeRecord {
            bridge_version: "0.2".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp,
            parent_hash: last_hash.clone(),
            project_id: record_project_id.to_string(),
            session_id: Some(record_session_id.to_string()),
            tx_id: None,
            record_kind: "insight".to_string(),
            payload,
            privacy,
        };

        // Compute current record hash for next record's parent_hash
        use sha2::{Digest, Sha256};
        let json_for_hash = serde_json::to_string(&record).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_for_hash.as_bytes());
        let hash_hex = format!("{:x}", hasher.finalize());
        last_hash = Some(hash_hex.clone());

        out_records.push(record);
    }

    if let Some(hash) = last_hash {
        event_store.set_sync_state("last_outbound_hash", &hash)?;
    }

    if out_records.is_empty() {
        println!("No insights to push.");
        return Ok(());
    }

    let temp_dir = std::env::temp_dir();
    let export_path = temp_dir.join("aibrains_export.ndjson");
    let mut file = std::fs::File::create(&export_path)?;

    for record in out_records {
        let json = serde_json::to_string(&record)?;
        writeln!(file, "{}", json)?;
    }
    file.flush()?;

    println!("Triggering ChangeGuard bridge import...");
    let output = std::process::Command::new("changeguard")
        .args([
            "bridge",
            "import",
            "--from",
            export_path.to_string_lossy().as_ref(),
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            println!("{}", String::from_utf8_lossy(&out.stdout));
            println!("Successfully pushed insights to ChangeGuard.");
        }
        Ok(out) => {
            eprintln!(
                "ChangeGuard import failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Err(e) => {
            println!(
                "ChangeGuard CLI not found or failed to execute. Error: {}",
                e
            );
        }
    }

    Ok(())
}

#[allow(clippy::disallowed_methods)]
pub fn run_query(
    ctx: &AppContext,
    query: String,
    format: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let fmt = format.unwrap_or_else(|| "pretty".to_string());
    if fmt == "ndjson" {
        #[cfg(feature = "graph")]
        let graph_vault = ai_brains_graph::GraphVault::new((*ctx.conn).clone());
        #[cfg(feature = "graph")]
        let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));
        #[cfg(not(feature = "graph"))]
        let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

        let project_id_str =
            std::env::var("AI_BRAINS_PROJECT_ID").unwrap_or_else(|_| "default-project".to_string());
        let session_id_str = std::env::var("AI_BRAINS_SESSION_ID")
            .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new().to_string());

        use std::str::FromStr;
        let project_id = ai_brains_core::ids::ProjectId::from_str(&project_id_str)
            .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new());
        let session_id = ai_brains_core::ids::SessionId::from_str(&session_id_str)
            .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new());

        let hits = ai_brains_retrieval::recall(
            &ctx.conn,
            graph_search.as_ref(),
            &query,
            5,
            Some(project_id),
            Some(session_id),
        )?;

        use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| String::new());

        for h in hits {
            let payload = serde_json::json!({
                "type": "Insight",
                "memory_id": h.memory_id,
                "relevance": h.score.unwrap_or(1.0),
                "content": h.content
            });

            let record = BridgeRecord {
                bridge_version: "0.2".to_string(),
                direction: BridgeDirection::Outbound,
                timestamp: timestamp.clone(),
                parent_hash: None,
                project_id: project_id.to_string(),
                session_id: Some(session_id.to_string()),
                tx_id: None,
                record_kind: "insight".to_string(),
                payload,
                privacy: ai_brains_core::privacy::Privacy::LocalOnly,
            };

            let json = serde_json::to_string(&record)?;
            println!("{}", json);
        }
        return Ok(());
    }

    println!("--- AI-Brains Recall ---");
    // 1. Local Recall
    crate::commands::recall::run(ctx, query.clone(), 3, None, None, fmt)?;

    println!("\n--- ChangeGuard Ledger Search ---");
    // 2. ChangeGuard Query (Attempt to call CLI)
    let output = std::process::Command::new("changeguard")
        .args(["ledger", "search", &query])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            println!("{}", String::from_utf8_lossy(&out.stdout));
        }
        Ok(out) => {
            eprintln!(
                "ChangeGuard search failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Err(_) => {
            println!("ChangeGuard CLI not found or failed to execute.");
        }
    }

    Ok(())
}
