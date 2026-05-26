use crate::context::AppContext;
use ai_brains_contracts::preflight::PreflightContextResponse;
use ai_brains_core::ids::ProjectId;
use ai_brains_retrieval::build_preflight;
use is_terminal::IsTerminal;

pub fn run(
    ctx: &AppContext,
    max_words: usize,
    project_id: Option<ProjectId>,
    pretty: bool,
    format: Option<String>,
    scope: Vec<String>,
    summary: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new(ctx.conn.clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let scope_paths = if scope.is_empty() {
        None
    } else {
        Some(normalize_scope_paths(&scope))
    };

    let context = build_preflight(
        &ctx.conn,
        graph_search.as_ref(),
        max_words,
        project_id,
        scope_paths,
    )?;

    if summary {
        print_summary(ctx, project_id, &context.text);
        return Ok(());
    }

    // Smart defaulting: If stdout is a TTY and no format is specified, use human mode.
    let is_tty = std::io::stdout().is_terminal();
    let format_str = format.unwrap_or_else(|| {
        if is_tty {
            "human".to_string()
        } else {
            "json".to_string()
        }
    });

    let human_mode = pretty
        || format_str.eq_ignore_ascii_case("human")
        || format_str.eq_ignore_ascii_case("pretty");

    if human_mode {
        println!("{}", context.text);
    } else {
        let response = PreflightContextResponse {
            text: context.text,
            word_count: context.word_count,
        };
        println!("{}", serde_json::to_string(&response)?);
    }
    Ok(())
}

fn print_summary(_ctx: &AppContext, project_id: Option<ProjectId>, text: &str) {
    let project_name = project_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "global".to_string());

    println!("--- AI-Brains Preflight Summary ---");
    println!("Project: {}", project_name);

    // Heuristic counts based on markers in context text
    let hotspot_count = text.matches("HOTSPOT:").count();
    let decision_count = text.matches("DECISION:").count();
    let constraint_count = text.matches("CONSTRAINT:").count();
    let session_count = text.matches("Session ID:").count();

    println!("Hotspots: {}", hotspot_count);
    println!("Decisions: {}", decision_count);
    println!("Constraints: {}", constraint_count);
    println!("Active Sessions: {}", session_count);
    println!("Total Word Count: {}", text.split_whitespace().count());
    println!("\nUse --pretty or --format json for full context.");
}

/// Normalize scope paths for Windows: resolve drive case, UNC prefixes, separator consistency.
fn normalize_scope_paths(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                return None;
            }
            let normalized = std::path::Path::new(trimmed);
            if normalized.exists() {
                Some(
                    std::fs::canonicalize(normalized)
                        .ok()
                        .and_then(|pb| pb.to_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| trimmed.to_string()),
                )
            } else {
                Some(trimmed.replace('\\', "/").to_lowercase())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_scope_paths_filters_empty() {
        let paths = vec![
            "  ".to_string(),
            "".to_string(),
            "nonexistent/file.rs".to_string(),
        ];
        let normalized = normalize_scope_paths(&paths);
        assert_eq!(normalized.len(), 1);
        // Non-existent paths get lowercased with forward slashes
        assert!(normalized[0].contains("nonexistent/file.rs"));
    }

    #[test]
    fn normalize_scope_paths_normalizes_separators() {
        let paths = vec!["C:\\dev\\src\\lib.rs".to_string()];
        let normalized = normalize_scope_paths(&paths);
        assert_eq!(normalized.len(), 1);
        // Non-existent path: should be lowercased with forward slashes
        let result = &normalized[0];
        assert!(
            !result.contains('\\'),
            "Backslashes should be normalized: {}",
            result
        );
    }

    #[test]
    fn normalize_scope_paths_handles_existing_path() {
        // Use a path we know exists (the project directory)
        let paths = vec!["C:\\dev\\AI-Brains\\src".to_string()];
        let normalized = normalize_scope_paths(&paths);
        assert_eq!(normalized.len(), 1);
        // Canonicalization should produce a valid path string
        assert!(!normalized[0].is_empty());
    }
}
