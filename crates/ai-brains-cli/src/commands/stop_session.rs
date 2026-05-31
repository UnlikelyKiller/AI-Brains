use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService, SessionStopCommand, SessionStopStatus};
use ai_brains_core::ids::{HarnessId, SessionId};
use ai_brains_core::privacy::Privacy;
use std::str::FromStr;

pub fn run(ctx: &AppContext, session_id_str: String) -> Result<(), Box<dyn std::error::Error>> {
    let session_id = SessionId::from_str(&session_id_str)?;
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

    service.stop_session(
        SessionStopCommand {
            session_id,
            harness_id: HarnessId::new(), // In a real scenario, this should come from context
            privacy: Privacy::LocalOnly,
            status: SessionStopStatus::Completed,
            reason: None,
        },
        capture_context,
        &mut sink,
    )?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to stop session: {}", err).into());
    }

    println!("Session {} marked as completed.", session_id);
    Ok(())
}
