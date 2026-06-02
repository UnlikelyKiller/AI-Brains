use crate::context::AppContext;
use ai_brains_contracts::recall::{RecallResponse, RecallResult};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, EventKind, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{recall, RecallOptions};
use ai_brains_store::EventStore;
use std::str::FromStr;

pub struct RecallRunOptions {
    pub query: String,
    pub limit: usize,
    pub project_id: Option<ProjectId>,
    pub session_id: Option<SessionId>,
    pub format: String,
    pub semantic: bool,
    pub graph_boost: f64,
    pub graph_hop_depth: usize,
    pub quiet: bool,
}

pub fn run(ctx: &AppContext, options: RecallRunOptions) -> Result<(), Box<dyn std::error::Error>> {
    let effective_session_id = options.session_id.or_else(|| {
        let generated = SessionId::new();
        eprintln!(
            "No session id supplied for recall; using generated session {} for graph provenance.",
            generated
        );
        Some(generated)
    });

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
        &options.query,
        options.limit,
        RecallOptions {
            project_id: options.project_id,
            session_id: options.session_id,
            semantic: options.semantic,
            graph_boost: options.graph_boost,
            graph_hop_depth: options.graph_hop_depth,
            quiet: options.quiet,
        },
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
                session_id: effective_session_id,
                project_id: options.project_id,
                tx_id: None,
                rank: Some(rank as u32),
                source_tag: Some(hit.source.clone()),
                query_text: Some(options.query.clone()),
            }));
            if let Ok(ev) = ev {
                if let Err(e) = event_store.append_event(&ev) {
                    tracing::warn!(
                        "Failed to emit MemoryPinned event for {}: {}",
                        hit.memory_id,
                        e
                    );
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
            options.query
        );
    }

    match options.format.as_str() {
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
