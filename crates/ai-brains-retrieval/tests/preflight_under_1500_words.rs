mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::build_preflight;

#[test]
fn preflight_under_1500_words() -> Result<(), Box<dyn std::error::Error>> {
    let long_text = (0..1700)
        .map(|i| format!("word{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let store = common::store_with_memory(&long_text, Privacy::CloudOk)?;

    let preflight = build_preflight(store.connection(), None, 1500, None)?;
    assert!(preflight.word_count <= 1500);
    Ok(())
}
