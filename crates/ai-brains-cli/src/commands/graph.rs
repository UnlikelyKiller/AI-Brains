use crate::context::AppContext;
use ai_brains_graph::{GraphRebuilder, GraphSearch, GraphVault};
use ai_brains_store::SqliteEventStore;
use serde::Serialize;

#[derive(Serialize)]
struct NeighborsOutput<'a> {
    memory_id: &'a str,
    neighbors: Vec<ai_brains_graph::queries::NeighborHit>,
}

#[derive(Serialize)]
struct HierarchyOutput<'a> {
    root: &'a str,
    synthesized_from: Vec<String>,
}

#[derive(Serialize)]
struct SessionOutput<'a> {
    session_id: &'a str,
    memories: Vec<String>,
}

#[derive(Serialize)]
struct GraphHealthOutput<'a> {
    nodes: i64,
    edges: i64,
    status: &'a str,
    note: &'a str,
}

pub fn rebuild(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[graph] Starting graph rebuild...");

    let event_store = SqliteEventStore::new((*ctx.conn).clone());
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let rebuilder = GraphRebuilder::new(&graph_vault, &event_store);

    rebuilder.rebuild()?;

    eprintln!("[graph] Rebuild complete.");
    Ok(())
}

pub fn neighbors(ctx: &AppContext, memory_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let neighbors = searcher.get_neighbors(memory_id)?;

    let output = NeighborsOutput {
        memory_id,
        neighbors,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub fn hierarchy(ctx: &AppContext, memory_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let synthesized = searcher.get_synthesized_hierarchy(memory_id)?;

    let output = HierarchyOutput {
        root: memory_id,
        synthesized_from: synthesized,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub fn session(ctx: &AppContext, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let memories = searcher.get_session_memories(session_id)?;

    let output = SessionOutput {
        session_id,
        memories,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub fn update(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock vault: {}", e))?;

    let node_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM graph_node", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count graph nodes: {}", e))?;

    let edge_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM graph_edge", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count graph edges: {}", e))?;

    let report = GraphHealthOutput {
        nodes: node_count,
        edges: edge_count,
        status: "live",
        note: "Graph is updated incrementally on each event append. Use 'graph rebuild' for full resync.",
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
