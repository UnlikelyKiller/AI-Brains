mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::build_preflight;

#[test]
fn preflight_includes_active_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory("memory context", Privacy::CloudOk)?;
    let session_id = common::append_active_session(&store)?;

    let preflight = build_preflight(store.connection(), None, 1500, None)?;
    assert!(preflight.text.contains("--- Session:"));
    assert!(preflight.text.contains(&session_id));
    Ok(())
}
