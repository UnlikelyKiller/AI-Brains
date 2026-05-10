use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::import_antigravity_sessions;
use ai_brains_capture::CaptureService;
use ai_brains_core::ids::ProjectId;
use std::str::FromStr;

pub fn run(ctx: &AppContext, days: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for Antigravity sessions...");

    let service = CaptureService::new();
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
    };

    // Inherit project ID from environment if available
    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default();

    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let (total_turns, sessions_imported) = import_antigravity_sessions(
        query_store.as_ref(),
        &service,
        &mut sink,
        days,
        project_id,
    )?;

    if let Some(err) = sink.last_error {
        return Err(format!("Antigravity import encountered an error: {}", err).into());
    }

    if sessions_imported == 0 {
        println!("No new Antigravity sessions found to import.");
    } else {
        println!(
            "Antigravity import complete. Processed {} turn(s) from {} session(s).",
            total_turns, sessions_imported
        );
    }

    Ok(())
}
