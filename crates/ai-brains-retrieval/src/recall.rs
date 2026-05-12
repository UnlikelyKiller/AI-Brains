use crate::errors::Result;
use crate::lexical::lexical_search;
use crate::GraphSearch;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, PartialEq)]
pub struct RecallHit {
    pub memory_id: String,
    pub content: String,
    pub source: String,
    pub score: Option<f64>,
}

pub fn recall(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    query: &str,
    limit: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<RecallHit>> {
    let mut hits = lexical_search(conn, query, project_id, session_id)?
        .into_iter()
        .take(limit)
        .map(|memory| RecallHit {
            memory_id: memory.memory_id,
            content: memory.content,
            source: "fts".to_string(),
            score: memory.score,
        })
        .collect::<Vec<_>>();

    // If graph is available, we could augment here.
    // For now, we just pass the graph through to satisfy the contract.
    if let Some(_searcher) = graph {
        // Placeholder for future graph-based ranking/expansion
    }

    if hits.len() > limit {
        hits.truncate(limit);
    }

    Ok(hits)
}
