mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::{build_preflight, lexical_search};

#[test]
fn privacy_filter_excludes_sealed() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "-----BEGIN PRIVATE KEY----- x -----END PRIVATE KEY-----",
        Privacy::Sealed,
    )?;

    let results = lexical_search(store.connection(), "PRIVATE KEY", None, None)?;
    assert!(results.is_empty());

    let preflight = build_preflight(store.connection(), None, 1500, None)?;
    assert!(preflight.text.is_empty());
    Ok(())
}
