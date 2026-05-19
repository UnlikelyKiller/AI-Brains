mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::build_preflight;

#[test]
fn preflight_does_not_duplicate_safety_entries_in_index() -> Result<(), Box<dyn std::error::Error>>
{
    // Pin a memory matching the safety query pattern (HOTSPOT:)
    let content = "HOTSPOT: brittle file src/main.rs has high churn score 42";
    let store = common::store_with_memory(content, Privacy::CloudOk)?;

    let project_id = ai_brains_core::ids::ProjectId::from_uuid(uuid::Uuid::nil());
    let context = build_preflight(store.connection(), None, 1500, Some(project_id))?;

    // The content should appear exactly once (in the safety section),
    // NOT duplicated in the memory index or most-recent section
    let occurrences = context.text.matches("brittle file src/main.rs").count();
    assert_eq!(
        occurrences, 1,
        "HOTSPOT content should appear exactly once, but appeared {} times\nFull output:\n{}",
        occurrences, context.text
    );

    Ok(())
}
