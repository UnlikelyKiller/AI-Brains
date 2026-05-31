use crate::context::AppContext;
use ai_brains_contracts::recall::{RecallResponse, RecallResult};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, EventKind, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::recall;
use ai_brains_store::EventStore;
use std::str::FromStr;

pub fn run(
    ctx: &AppContext,
    query: String,
    limit: usize,
    project_id: Option<ProjectId>,
    session_id: Option<SessionId>,
    format: String,
    semantic: bool,
    graph_boost: f64,
    graph_hop_depth: usize,
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
        semantic,
        graph_boost,
        graph_hop_depth,
    )?;

    // Emit MemoryPinned events for each recall hit so the graph projector can
    // build session -> memory RECALLS edges.
    #[cfg(feature = "graph")]
    let event_store = crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone());
    #[cfg(not(feature = "graph"))]
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    for (rank, hit) in hits.iter().enumerate() {
        if let Ok(memory_id) = MemoryId::from_str(&hit.memory_id) {
            let ev = EventBuilder::new(
                AggregateType::Memory,
                memory_id.as_uuid(),
                EventKind::MemoryPinned,
                Actor::System,
                ai_brains_core::privacy::Privacy::LocalOnly,
            )
            .build(Payload::MemoryPinned(MemoryPinnedPayload {
                memory_id,
                content: hit.content.clone(),
                session_id,
                project_id,
                tx_id: None,
                rank: Some(rank as u32),
                source_tag: Some(hit.source.clone()),
                query_text: Some(query.clone()),
            }));
            if let Ok(ev) = ev {
                if let Err(e) = event_store.append_event(&ev) {
                    tracing::warn!("Failed to emit MemoryPinned event for {}: {}", hit.memory_id, e);
                }
            }
        }
    }

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
