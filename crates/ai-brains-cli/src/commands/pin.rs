use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;

pub fn run(
    ctx: &AppContext,
    content: String,
    role: String,
    privacy_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .map_err(|_| "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first.")?
        .parse::<ProjectId>()?;

    let session_id = std::env::var("AI_BRAINS_SESSION_ID")
        .map_err(|_| "AI_BRAINS_SESSION_ID not set. Run 'ai-brains context' first.")?
        .parse::<SessionId>()?;

    let harness_id = std::env::var("AI_BRAINS_HARNESS_ID")
        .ok()
        .and_then(|s| s.parse::<HarnessId>().ok())
        .unwrap_or_default();

    let privacy = match privacy_str.to_lowercase().as_str() {
        "cloudok" => Privacy::CloudOk,
        "localonly" => Privacy::LocalOnly,
        "neverinject" => Privacy::NeverInject,
        "sealed" => Privacy::Sealed,
        _ => Privacy::LocalOnly,
    };

    let request = IngestRequest {
        session_id,
        project_id,
        harness_id,
        turn_id: TurnId::new(),
        role,
        content,
        thinking: None,
        privacy,
    };

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        project_id,
        session_id,
        harness_id,
        privacy,
    )?;

    if let Some(err) = sink.last_error.take() {
        return Err(format!("Failed to auto-initialize context: {}", err).into());
    }

    service.ingest_request(request, capture_context, &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to pin memory: {}", err).into());
    }

    println!("Memory successfully pinned to vault.");
    Ok(())
}
