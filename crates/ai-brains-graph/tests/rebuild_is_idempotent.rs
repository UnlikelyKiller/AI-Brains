mod common;

use ai_brains_graph::{GraphRebuilder, GraphVault};

#[test]
fn test_rebuild_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (_session_id, _project_id) = common::append_session(&store)?;

    let conn = store.connection().clone();
    let vault = GraphVault::new(conn);

    let rebuilder = GraphRebuilder::new(&vault, &store);

    // First rebuild
    rebuilder.rebuild()?;

    // Second rebuild should not error and should be idempotent
    rebuilder.rebuild()?;

    Ok(())
}
