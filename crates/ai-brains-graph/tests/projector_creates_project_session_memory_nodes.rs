mod common;

use ai_brains_graph::GraphProjector;
use ai_brains_store::EventStore;

#[test]
fn test_projector_creates_nodes_and_edges() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (session_id, project_id) = common::append_session(&store)?;

    // In Track T29, the graph and store share the same connection
    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(ai_brains_graph::SqliteGraphBackend::new(conn.clone()));
    let mut projector = GraphProjector::new(backend);

    // Replay events through projector
    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;

    // Verify nodes and edges via SQL
    let conn_lock = conn.lock().map_err(|e| e.to_string())?;

    // Check Project Node
    let name: String = conn_lock.query_row(
        "SELECT n.kind FROM graph_node n WHERE n.external_id = ?",
        [project_id.clone()],
        |row| row.get(0),
    )?;
    assert_eq!(name, "project");

    // Check Session link to Project
    let count: i64 = conn_lock.query_row(
        "SELECT count(*) FROM graph_edge e 
         JOIN graph_node s ON e.src_id = s.node_id 
         JOIN graph_node p ON e.dst_id = p.node_id 
         WHERE s.external_id = ? AND p.external_id = ? AND e.label = 'IN_PROJECT'",
        [session_id, project_id],
        |row| row.get(0),
    )?;
    assert_eq!(count, 1);

    Ok(())
}
