mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::recall;

#[test]
fn recall_returns_sources() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "unique lexical retrieval source test marker",
        Privacy::CloudOk,
    )?;

    let hits = recall(
        store.connection(),
        None,
        "unique lexical retrieval source test marker",
        100,
        None,
        None,
    )?;
    assert!(!hits.is_empty());
    // Find the one from our local store
    let local_hit = hits
        .iter()
        .find(|h| h.source == "fts")
        .expect("Should find FTS hit");
    assert!(local_hit
        .content
        .contains("unique lexical retrieval source test marker"));
    Ok(())
}
