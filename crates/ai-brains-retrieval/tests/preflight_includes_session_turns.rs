mod common;

use ai_brains_retrieval::build_preflight;

#[test]
fn preflight_includes_session_turns() -> Result<(), Box<dyn std::error::Error>> {
    let store =
        common::store_with_memory("pinned memory", ai_brains_core::privacy::Privacy::CloudOk)?;
    let (session_id, project_id) = common::append_active_session(&store)?;

    common::append_turn(
        &store,
        &session_id.to_string(),
        "user",
        "What is the capital of France?",
    )?;
    common::append_turn(
        &store,
        &session_id.to_string(),
        "assistant",
        "The capital of France is Paris.",
    )?;

    let context = build_preflight(store.connection(), None, 1500, Some(project_id))?;

    // This is expected to fail currently as preflight.rs only lists active session IDs
    assert!(context.text.contains("What is the capital of France?"));
    assert!(context.text.contains("The capital of France is Paris."));
    Ok(())
}
