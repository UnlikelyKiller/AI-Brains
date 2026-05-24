use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::agy::{generate_deterministic_turn_id, parse_agy_transcript};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgyHookPayload {
    pub transcript_path: String,
    pub session_id: String,
    pub project_hash: String,
}

pub fn run(ctx: &AppContext, payload_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload: AgyHookPayload = serde_json::from_str(payload_json)?;
    let transcript_path = PathBuf::from(&payload.transcript_path);

    // Normalize path via ai-brains-path
    let normalized_path =
        ai_brains_path::normalize_project_path(&transcript_path.to_string_lossy())?;

    let session_id = SessionId::from_uuid(
        uuid::Uuid::parse_str(&payload.session_id)
            .map_err(|e| format!("Invalid session ID in agy hook: {}", e))?,
    );

    // Canonical agy Harness ID (new one)
    let agy_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000002")?;

    // Phase 1: Resolve Project ID from agy projectHash (Requirement T48.4)
    let original_resolved_pid = ctx.resolve_project_id_from_alias(&payload.project_hash)?;
    let mut project_id = original_resolved_pid;

    // Phase 2: Fallback to environment PID if not resolved (Requirement T48.2)
    if project_id.is_none() {
        if let Ok(env_pid_str) = std::env::var("AI_BRAINS_PROJECT_ID") {
            if let Ok(env_pid) = ProjectId::from_str(&env_pid_str) {
                project_id = Some(env_pid);
            }
        }
    }

    let project_id = project_id.unwrap_or_else(ProjectId::new);

    let turns = parse_agy_transcript(std::path::Path::new(normalized_path.canonical()))?;

    // Phase 3: Filter ingestable turns (Requirement Capture-Privacy)
    let ingestable_turns: Vec<_> = turns
        .into_iter()
        .filter(|t| t.role == "user" || t.role == "assistant")
        .collect();

    // Phase 4: Delta Sync (Requirement T49.1)
    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let max_turn = query_store
        .get_max_turn_index(&session_id)
        .map_err(|e| format!("Failed to query vault turn state: {}", e))?;
    let next_index = max_turn.map(|m| m + 1).unwrap_or(0);

    if ingestable_turns.len() <= next_index as usize {
        println!("No new turns to ingest (vault already has {}).", next_index);
        return Ok(());
    }

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    // Phase 5: Ensure project/session exists (MUST be before alias linking)
    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        project_id,
        session_id,
        agy_harness,
        Privacy::LocalOnly,
    )?;

    // Phase 6: Auto-link alias to project (Requirement T48.2)
    ctx.ensure_project_alias(
        &mut sink,
        project_id,
        payload.project_hash.clone(),
        Privacy::LocalOnly,
    )?;
    if original_resolved_pid.is_none() {
        println!(
            "Auto-linked projectHash {} to project {}",
            payload.project_hash, project_id
        );
    }

    let mut turn_count = 0;
    for (i, turn) in ingestable_turns
        .iter()
        .enumerate()
        .skip(next_index as usize)
    {
        let turn_id = generate_deterministic_turn_id(&session_id, i);

        let request = IngestRequest {
            session_id,
            project_id,
            harness_id: agy_harness,
            turn_id,
            role: turn.role.clone(),
            content: turn.content.clone(),
            privacy: Privacy::LocalOnly,
            thinking: None,
            tx_id: None,
        };

        service.ingest_request(request, capture_context.clone(), &mut sink)?;
        turn_count += 1;
    }

    println!(
        "Successfully ingested {} turns from agy transcript.",
        turn_count
    );
    Ok(())
}
