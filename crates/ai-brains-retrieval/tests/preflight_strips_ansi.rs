mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::build_preflight;

#[test]
fn preflight_strips_ansi_from_pinned_memories() -> Result<(), Box<dyn std::error::Error>> {
    // Pin a memory containing ANSI escape codes (simulating raw ChangeGuard output)
    let ansi_content =
        "HOTSPOT: Brittle files identified by ChangeGuard:\n\n\u{1b}[32m INFO\u{1b}[0m file loaded";
    let store = common::store_with_memory(ansi_content, Privacy::CloudOk)?;

    let context = build_preflight(store.connection(), None, 1500, None)?;

    // The preflight output must NOT contain ANSI escape sequences
    assert!(
        !context.text.contains('\x1b'),
        "Preflight output should not contain ANSI escape sequences, but found: {:?}",
        context.text
    );
    // The cleaned text SHOULD still contain the meaningful content
    assert!(
        context.text.contains("HOTSPOT"),
        "Preflight output should contain 'HOTSPOT' after ANSI stripping"
    );

    Ok(())
}
