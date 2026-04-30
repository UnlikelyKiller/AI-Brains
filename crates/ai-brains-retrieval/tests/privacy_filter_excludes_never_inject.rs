mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::{build_preflight, lexical_search};

#[test]
fn privacy_filter_excludes_never_inject() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory("sensitive operator note", Privacy::NeverInject)?;

    let results = lexical_search(store.connection(), "operator", None, None)?;
    assert!(results.is_empty());

    let preflight = build_preflight(store.connection(), None, 1500, None)?;
    assert!(preflight.text.is_empty());
    Ok(())
}
