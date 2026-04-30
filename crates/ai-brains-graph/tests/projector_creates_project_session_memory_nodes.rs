mod common;

use ai_brains_graph::{GraphProjector, LadybugVault};
use ai_brains_store::EventStore;
use tempfile::tempdir;

#[test]
fn test_projector_creates_nodes_and_edges() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (session_id, project_id) = common::append_session(&store)?;

    let temp_dir = tempdir()?;
    let vault_path = temp_dir.path().join("graph.db");
    let vault = LadybugVault::open(&vault_path)?;
    let projector = GraphProjector::new(&vault);

    // Replay events through projector
    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }

    // Verify nodes and edges via query
    let conn = vault.connection()?;

    // Check Project
    let mut res = conn.query(&format!(
        "MATCH (p:Project {{id: '{}'}}) RETURN p.name",
        project_id
    ))?;
    assert!(res.has_next());
    assert_eq!(
        res.get_next()?.get_column(0).map(|v| v.to_string()),
        Some("test-project".to_string())
    );

    // Check Session link to Project
    let mut res = conn.query(&format!(
        "MATCH (s:Session {{id: '{}'}})-[:IN_PROJECT]->(p:Project {{id: '{}'}}) RETURN count(*)",
        session_id, project_id
    ))?;
    assert!(res.has_next());
    assert_eq!(
        res.get_next()?.get_column(0).map(|v| v.to_string()),
        Some("1".to_string())
    );

    Ok(())
}
