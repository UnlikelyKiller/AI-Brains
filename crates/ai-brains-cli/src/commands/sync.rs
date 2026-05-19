use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::TurnId;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn run_pull(ctx: &AppContext, from_file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !from_file.exists() {
        return Err(format!("File not found: {}", from_file.display()).into());
    }

    let file = File::open(from_file)?;
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
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(err) => {
                eprintln!("Failed to parse BridgeRecord: {}. Skipping line.", err);
                continue;
            }
        };

        // Pulling only inbound records (ChangeGuard -> AI-Brains)
        if record.direction != ai_brains_contracts::bridge::BridgeDirection::Inbound {
            continue;
        }

        // Ensure context exists
        ctx.ensure_project_and_session_exists(
            &mut sink,
            &service,
            &capture_context,
            record.project_id,
            record.session_id,
            ai_brains_core::ids::HarnessId::default(), // Unknown harness
            record.privacy,
        )?;

        // Map record to IngestRequest
        // Note: record_kind might indicate role or type
        let role = match record.record_kind.to_lowercase().as_str() {
            "user" | "prompt" => "user",
            "assistant" | "response" | "final" => "assistant",
            _ => "assistant", // Default to assistant for external signals
        };

        // Flatten payload to string if it's not already
        let content = if let Some(s) = record.payload.as_str() {
            s.to_string()
        } else {
            record.payload.to_string()
        };

        let request = IngestRequest {
            session_id: record.session_id,
            project_id: record.project_id,
            harness_id: ai_brains_core::ids::HarnessId::default(),
            turn_id: TurnId::new(),
            role: role.to_string(),
            content,
            thinking: None,
            privacy: record.privacy,
            tx_id: record.tx_id,
        };

        service.ingest_request(request, capture_context.clone(), &mut sink)?;
        count += 1;
    }

    println!("Successfully synced {} records from file.", count);
    Ok(())
}
