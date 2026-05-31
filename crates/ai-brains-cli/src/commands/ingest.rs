use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{parse_ingest_request, CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestResponse;
use std::io::{self, Read};

pub fn run(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request = parse_ingest_request(&input)?;

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(std::sync::Arc::clone(&ctx.conn))),
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        request.project_id,
        request.session_id,
        request.harness_id,
        request.privacy,
    )?;

    if let Some(err) = sink.last_error.take() {
        return Err(format!("Failed to auto-initialize context: {}", err).into());
    }

    let outcome = service.ingest_request(request, capture_context, &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist turn: {}", err).into());
    }

    let response = IngestResponse {
        event_id: outcome.events[0].event_id.to_string(),
        processed: true,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
