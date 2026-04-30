mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::recall;

#[test]
fn recall_returns_sources() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory("lexical retrieval source test", Privacy::CloudOk)?;

    let hits = recall(store.connection(), None, "retrieval source", 5, None, None)?;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].source, "fts");
    Ok(())
}
