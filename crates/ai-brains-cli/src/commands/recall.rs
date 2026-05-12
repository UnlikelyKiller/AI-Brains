use crate::context::AppContext;
use ai_brains_contracts::recall::{RecallResponse, RecallResult};
use ai_brains_core::ids::{ProjectId, SessionId};
use ai_brains_retrieval::recall;

pub fn run(
    ctx: &AppContext,
    query: String,
    limit: usize,
    project_id: Option<ProjectId>,
    session_id: Option<SessionId>,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new((*ctx.conn).clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let hits = recall(
        &ctx.conn,
        graph_search.as_ref(),
        &query,
        limit,
        project_id,
        session_id,
    )?;

    let response = RecallResponse {
        results: hits
            .into_iter()
            .map(|h| RecallResult {
                memory_id: h.memory_id,
                content: h.content,
                source: h.source,
                score: h.score,
            })
            .collect(),
    };

    if response.results.is_empty() {
        eprintln!(
            "No results for '{}'. Try shorter terms or check spelling.",
            query
        );
    }

    match format.as_str() {
        "pretty" => {
            for r in &response.results {
                if let Some(s) = r.score {
                    println!("[score={:.3}] {}: {}", s, r.memory_id, r.content);
                } else {
                    println!("{}: {}", r.memory_id, r.content);
                }
            }
        }
        _ => println!("{}", serde_json::to_string(&response)?),
    }

    Ok(())
}
