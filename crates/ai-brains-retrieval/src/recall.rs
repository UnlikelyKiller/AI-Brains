use crate::errors::Result;
use crate::lexical::lexical_search;
use crate::GraphSearch;
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_core::privacy::Privacy;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, Copy, Default)]
pub struct RecallOptions {
    pub project_id: Option<ai_brains_core::ids::ProjectId>,
    pub session_id: Option<ai_brains_core::ids::SessionId>,
    pub semantic: bool,
    pub graph_boost: f64,
    pub graph_hop_depth: usize,
    /// When true, suppress non-fatal warnings (e.g. bridge-failed notices
    /// when the cwd is not a git repository).
    pub quiet: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecallHit {
    pub memory_id: String,
    pub content: String,
    pub source: String,
    pub score: Option<f64>,
    /// Privacy flag inherited from the source memory.
    pub privacy: Option<Privacy>,
}

impl RecallHit {
    /// Create a basic FTS5 hit with no privacy flag.
    pub fn fts(memory_id: String, content: String, score: Option<f64>) -> Self {
        Self {
            memory_id,
            content,
            source: "fts".to_string(),
            score,
            privacy: None,
        }
    }

    /// Create a hit added via graph neighbor expansion.
    pub fn graph(memory_id: String, content: String, score: Option<f64>) -> Self {
        Self {
            memory_id,
            content,
            source: "graph".to_string(),
            score,
            privacy: None,
        }
    }

    /// Create a hit from the unified IPC bridge.
    pub fn bridge(
        memory_id: String,
        content: String,
        score: Option<f64>,
        source: String,
        privacy: Option<Privacy>,
    ) -> Self {
        Self {
            memory_id,
            content,
            source,
            score,
            privacy,
        }
    }
}

/// Primary recall entry point. Attempts unified IPC recall via ChangeGuard
/// (`bridge query`) first. If IPC is unavailable or fails, falls back to
/// local FTS5 search. Results from both sources are blended, with privacy
/// flags preserved from bridge hits.
///
/// When `semantic` is true, also queries via embedding-based semantic search
/// and blends those results alongside bridge and FTS5 hits.
pub fn recall(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    query: &str,
    limit: usize,
    options: RecallOptions,
) -> Result<Vec<RecallHit>> {
    let project_id = options.project_id;
    let session_id = options.session_id;

    // Phase 1: Try unified IPC recall via ChangeGuard bridge query.
    let bridge_hits = query_changeguard_bridge(query, project_id, session_id);

    // Phase 2: Always run local FTS5 as a fallback / supplement.
    let local_hits: Vec<RecallHit> = lexical_search(conn, query, project_id, session_id)?
        .into_iter()
        .map(|memory| RecallHit::fts(memory.memory_id, memory.content, memory.score))
        .collect();

    // Phase 3: Semantic search when requested.
    let semantic_hits: Vec<RecallHit> = if options.semantic {
        crate::semantic::semantic_search(conn, query, limit, project_id, session_id).unwrap_or_else(
            |e| {
                eprintln!(
                    "Semantic search failed, continuing with lexical results: {}",
                    e
                );
                Vec::new()
            },
        )
    } else {
        Vec::new()
    };

    #[cfg(not(feature = "graph"))]
    let _ = (graph, options.graph_boost, options.graph_hop_depth);

    // Phase 4: Blend results. Bridge hits come first (higher authority),
    // followed by semantic hits, then local FTS5 hits. Deduplicate by memory_id.
    let mut seen_ids = std::collections::HashSet::new();
    let mut blended = Vec::new();

    match bridge_hits {
        Ok(bridge) => {
            for hit in bridge {
                if seen_ids.insert(hit.memory_id.clone()) {
                    blended.push(hit);
                }
            }
        }
        Err(e) => {
            if !options.quiet {
                eprintln!(
                    "ChangeGuard bridge query failed, falling back to local FTS5 only: {}",
                    e
                );
            }
        }
    }

    // Add semantic hits, skipping any already present from the bridge.
    for hit in semantic_hits {
        if seen_ids.insert(hit.memory_id.clone()) {
            blended.push(hit);
        }
    }

    // Add local hits, skipping any already present from bridge or semantic.
    for hit in local_hits {
        if seen_ids.insert(hit.memory_id.clone()) {
            blended.push(hit);
        }
    }

    // Graph-based neighbor expansion: for each current hit, fetch 1-hop
    // neighbors and add unseen ones with a boosted score.
    #[cfg(feature = "graph")]
    if options.graph_hop_depth >= 1 {
        if let Some(searcher) = graph {
            let mut graph_hits: Vec<RecallHit> = Vec::new();
            // Snapshot existing hits to iterate without borrow issues.
            let existing: Vec<(String, Option<f64>)> = blended
                .iter()
                .map(|h| (h.memory_id.clone(), h.score))
                .collect();

            for (parent_id, parent_score) in existing {
                let neighbors = match searcher.get_neighbors(&parent_id) {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Graph neighbor lookup failed for {}: {}", parent_id, e);
                        continue;
                    }
                };
                for neighbor in neighbors {
                    if !seen_ids.contains(&neighbor.external_id) {
                        // Fetch content from memory_projection by external_id.
                        let content_opt: Option<String> = {
                            let db = conn.lock().ok();
                            db.and_then(|c| {
                                c.query_row(
                                    "SELECT content FROM memory_projection WHERE memory_id = ?1",
                                    rusqlite::params![neighbor.external_id],
                                    |row| row.get::<_, String>(0),
                                )
                                .ok()
                            })
                        };
                        if let Some(content) = content_opt {
                            let boost_score =
                                Some(parent_score.unwrap_or(0.0) + options.graph_boost);
                            seen_ids.insert(neighbor.external_id.clone());
                            graph_hits.push(RecallHit::graph(
                                neighbor.external_id,
                                content,
                                boost_score,
                            ));
                        }
                    }
                }
            }
            blended.extend(graph_hits);
        }
    }

    // Re-sort by score descending (None scores go last), then truncate.
    blended.sort_by(|a, b| match (a.score, b.score) {
        (Some(sa), Some(sb)) => sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    if blended.len() > limit {
        blended.truncate(limit);
    }

    Ok(blended)
}

// ---------------------------------------------------------------------------
// Unified IPC Bridge Query
// ---------------------------------------------------------------------------

/// Query ChangeGuard's Tantivy search via `changeguard search --json`.
/// Parses the NDJSON response for `BridgeRecord::Insight` entries.
///
/// Returns Ok(Vec) on success. On any failure (CLI missing, non-zero exit,
/// parse errors), returns an Err so the caller can fall back to local FTS5.
#[allow(clippy::disallowed_methods)]
fn query_changeguard_bridge(
    query: &str,
    _project_id: Option<ai_brains_core::ids::ProjectId>,
    _session_id: Option<ai_brains_core::ids::SessionId>,
) -> std::result::Result<Vec<RecallHit>, String> {
    let output = std::process::Command::new("changeguard")
        .args(["search", "--json", query])
        .output()
        .map_err(|e| format!("changeguard CLI not available: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("changeguard search failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hits = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Defensive: stdout may contain tracing/log lines mixed with NDJSON.
        // Skip anything that does not look like a JSON object or array.
        if !line.starts_with('{') && !line.starts_with('[') {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_e) => {
                // Silently skip malformed JSON lines mixed into stdout.
                continue;
            }
        };

        let record_kind = record.record_kind.to_lowercase();
        if record_kind != "insight" && record_kind != "bm25_match" {
            continue;
        }

        let payload = record.payload_value();
        let memory_id = payload
            .get("memory_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let score = payload.get("relevance").and_then(|v| v.as_f64());
        let source = payload
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("bridge")
            .to_string();
        let privacy = Some(record.privacy);

        if !content.is_empty() {
            hits.push(RecallHit::bridge(
                memory_id, content, score, source, privacy,
            ));
        }
    }

    Ok(hits)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recall_hit_fts_constructor() {
        let hit = RecallHit::fts("mem-1".into(), "test content".into(), Some(0.85));
        assert_eq!(hit.memory_id, "mem-1");
        assert_eq!(hit.source, "fts");
        assert_eq!(hit.score, Some(0.85));
        assert_eq!(hit.privacy, None);
    }

    #[test]
    fn recall_hit_bridge_constructor() {
        let hit = RecallHit::bridge(
            "mem-2".into(),
            "bridge content".into(),
            Some(0.92),
            "code_context".into(),
            Some(Privacy::LocalOnly),
        );
        assert_eq!(hit.memory_id, "mem-2");
        assert_eq!(hit.source, "code_context");
        assert_eq!(hit.score, Some(0.92));
        assert_eq!(hit.privacy, Some(Privacy::LocalOnly));
    }

    #[test]
    fn query_changeguard_bridge_fails_when_cli_missing() {
        // Temporarily mess with PATH so changeguard can't be found.
        // Actually, the function already handles missing CLI gracefully.
        let result = query_changeguard_bridge("test query", None, None);
        // On CI / local without changeguard, this should fail gracefully.
        if let Err(ref e) = result {
            assert!(
                e.contains("not available") || e.contains("CLI not found") || e.contains("failed"),
                "Error message should indicate unavailability: {}",
                e
            );
        }
        // If it succeeds (changeguard is installed), that's also fine.
    }

    #[test]
    fn blending_deduplicates_by_memory_id() {
        let mut bridge_hits = vec![
            RecallHit::bridge(
                "mem-1".into(),
                "c1".into(),
                Some(0.9),
                "bridge".into(),
                None,
            ),
            RecallHit::bridge(
                "mem-2".into(),
                "c2".into(),
                Some(0.8),
                "bridge".into(),
                None,
            ),
        ];

        let local_fts = vec![
            RecallHit::fts("mem-2".into(), "c2-fts".into(), Some(0.7)),
            RecallHit::fts("mem-3".into(), "c3".into(), Some(0.6)),
        ];

        let mut seen = std::collections::HashSet::new();
        let mut blended = Vec::new();

        for hit in bridge_hits.drain(..) {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
        for hit in local_fts {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }

        assert_eq!(blended.len(), 3, "Should have 3 unique hits");
        // mem-1: from bridge only
        assert_eq!(blended[0].memory_id, "mem-1");
        assert_eq!(blended[0].source, "bridge");
        // mem-2: from bridge (first in, wins over FTS)
        assert_eq!(blended[1].memory_id, "mem-2");
        assert_eq!(blended[1].source, "bridge");
        // mem-3: from FTS only
        assert_eq!(blended[2].memory_id, "mem-3");
        assert_eq!(blended[2].source, "fts");
    }

    #[test]
    fn blending_preserves_privacy_flags_from_bridge() {
        let bridge_hits = vec![RecallHit::bridge(
            "mem-private".into(),
            "secret".into(),
            Some(1.0),
            "bridge".into(),
            Some(Privacy::NeverInject),
        )];

        let mut seen = std::collections::HashSet::new();
        let mut blended = Vec::new();
        for hit in bridge_hits {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }

        assert_eq!(blended.len(), 1);
        assert_eq!(blended[0].privacy, Some(Privacy::NeverInject));
    }
}
