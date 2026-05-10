use crate::context::AppContext;
use ai_brains_contracts::preflight::PreflightContextResponse;
use ai_brains_core::ids::ProjectId;
use ai_brains_retrieval::build_preflight;

pub fn run(
    ctx: &AppContext,
    max_words: usize,
    project_id: Option<ProjectId>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new(ctx.conn.clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let context = build_preflight(&ctx.conn, graph_search.as_ref(), max_words, project_id)?;

    let response = PreflightContextResponse {
        text: context.text,
        word_count: context.word_count,
    };

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
