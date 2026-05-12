use std::collections::HashSet;

use crate::ansi::strip_ansi;
use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use crate::sessions::active_sessions;
use crate::word_budget::{trim_to_word_budget, word_count};
use crate::GraphSearch;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightContext {
    pub text: String,
    pub word_count: usize,
}

pub fn build_preflight(
    conn: &VaultConnection,
    _graph: Option<&GraphSearch>,
    max_words: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
) -> Result<PreflightContext> {
    let active = active_sessions(conn, project_id)?;
    let conn = conn.lock()?;

    let project_id_str: Option<String> = project_id.map(|p| p.to_string());

    let mut sections = Vec::new();

    // --- Onboarding & Safety Section (Max 15% of budget) ---
    let onboarding_budget = (max_words * 15) / 100;
    let mut safety_entries: Vec<(String, String)> = Vec::new(); // (content, updated_at)
    let mut safety_ids: HashSet<String> = HashSet::new();

    let safety_sql = if project_id_str.is_some() {
        "SELECT m.memory_id, m.content, m.updated_at FROM memory_projection m
         LEFT JOIN session_projection s ON m.session_id = s.session_id
         WHERE (m.content LIKE '%CONSTRAINT:%' OR m.content LIKE '%INVARIANT:%' OR m.content LIKE '%HOTSPOT:%')
         AND m.status = 'pinned'
         AND (s.project_id = ?1 OR m.project_id = ?1)
         ORDER BY m.updated_at DESC LIMIT 10"
    } else {
        "SELECT memory_id, content, updated_at FROM memory_projection
         WHERE (content LIKE '%CONSTRAINT:%' OR content LIKE '%INVARIANT:%' OR content LIKE '%HOTSPOT:%')
         AND status = 'pinned'
         ORDER BY updated_at DESC LIMIT 10"
    };

    let mut safety_stmt = conn.prepare(safety_sql)?;
    let mut safety_rows = if let Some(ref pid) = project_id_str {
        safety_stmt.query(rusqlite::params![pid, pid])?
    } else {
        safety_stmt.query([])?
    };
    while let Some(row) = safety_rows.next()? {
        let memory_id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let updated_at: String = row.get(2)?;
        safety_ids.insert(memory_id);
        safety_entries.push((strip_ansi(&content), updated_at));
    }

    // Deduplicate hotspot entries by file path: keep only the most recent score per path.
    // ORDER BY updated_at DESC ensures first occurrence is the freshest.
    let safety_entries = dedup_hotspots(safety_entries);

    if !safety_entries.is_empty() {
        let safety_text = format!(
            "--- Repository Bearings & Safety ---\n{}",
            safety_entries.join("\n\n")
        );
        sections.push(trim_to_word_budget(&safety_text, onboarding_budget));
    }

    if !active.is_empty() {
        let mut session_texts = Vec::new();
        for session in active {
            let mut session_lines = vec![format!("--- Session: {} ---", session.session_id)];
            for turn in session.turns {
                session_lines.push(format!("{}: {}", turn.role.to_uppercase(), turn.content));
            }
            session_texts.push(session_lines.join("\n"));
        }
        sections.push(session_texts.join("\n\n"));
    }

    // --- General Memory Index (scoped to current project when project_id is known) ---
    let index_sql = if project_id_str.is_some() {
        "SELECT m.memory_id, m.content, m.privacy, m.updated_at
         FROM memory_projection m
         LEFT JOIN session_projection s ON m.session_id = s.session_id
         WHERE m.status = 'pinned'
         AND (s.project_id = ?1 OR m.project_id = ?1)
         ORDER BY m.updated_at DESC"
    } else {
        "SELECT memory_id, content, privacy, updated_at
         FROM memory_projection
         WHERE status = 'pinned'
         ORDER BY updated_at DESC"
    };

    let mut stmt = conn.prepare(index_sql)?;
    let mut rows = if let Some(ref pid) = project_id_str {
        stmt.query(rusqlite::params![pid, pid])?
    } else {
        stmt.query([])?
    };
    let mut collected: Vec<(String, String)> = Vec::new(); // (content, updated_at)

    while let Some(row) = rows.next()? {
        let memory_id: String = row.get(0)?;
        let privacy: String = row.get(2)?;

        // Skip entries already shown in the safety section
        if safety_ids.contains(&memory_id) {
            continue;
        }

        if !is_injectable_privacy(&privacy) {
            continue;
        }

        let content: String = row.get(1)?;
        let updated_at: String = row.get(3)?;
        let content = strip_ansi(&content);
        let candidate = if collected.is_empty() {
            content.clone()
        } else {
            let mut parts: Vec<String> = collected.iter().map(|(c, _)| c.clone()).collect();
            parts.push(content.clone());
            parts.join("\n\n")
        };

        if word_count(&candidate) > max_words {
            break;
        }
        collected.push((content, updated_at));
    }

    if !collected.is_empty() {
        // 1. Build the index section with relative timestamps
        let mut index_lines = vec!["--- Memory Index (Briefing) ---".to_string()];
        for (i, (content, updated_at)) in collected.iter().enumerate() {
            let first_line = content.lines().next().unwrap_or("Untitled Memory");
            let summary = if first_line.len() > 60 {
                format!("{}...", &first_line[..57])
            } else {
                first_line.to_string()
            };
            let ts = relative_timestamp(updated_at);
            if ts.is_empty() {
                index_lines.push(format!("{}. {}", i + 1, summary));
            } else {
                index_lines.push(format!("{}. {} -- {}", i + 1, summary, ts));
            }
        }
        let index_text = index_lines.join("\n");

        // 2. Build the detailed section (only the most recent memory)
        let mut detailed_text = String::new();
        if let Some((most_recent, updated_at)) = collected.first() {
            let ts = relative_timestamp(updated_at);
            let header = if ts.is_empty() {
                "--- Most Recent Memory ---".to_string()
            } else {
                format!("--- Most Recent Memory (pinned {}) ---", ts)
            };
            detailed_text = format!(
                "{}\n\n{}\n\n(Use 'recall' to fetch details for other index items)",
                header, most_recent
            );
        }

        // 3. Assemble with budget awareness
        let remaining_budget = max_words.saturating_sub(word_count(&sections.join("\n\n")));
        let full_text = format!("{}\n\n{}", index_text, detailed_text);

        if word_count(&full_text) <= remaining_budget {
            sections.push(full_text);
        } else if word_count(&index_text) <= remaining_budget {
            sections.push(index_text);
        } else {
            sections.push(trim_to_word_budget(&index_text, remaining_budget));
            sections.push("... [Index Truncated]".to_string());
        }
    }

    let text = trim_to_word_budget(&sections.join("\n\n"), max_words);
    Ok(PreflightContext {
        word_count: word_count(&text),
        text,
    })
}

/// Extract file paths from hotspot table content (lines containing `| crates/` or similar).
fn extract_hotspot_paths(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            line.contains('|') && (line.contains("crates/") || line.contains("scripts/"))
        })
        .filter_map(|line| {
            // Last pipe-delimited field is the file path
            let parts: Vec<&str> = line.split('|').collect();
            parts.last().map(|s| s.trim().to_string())
        })
        .filter(|p| !p.is_empty() && !p.starts_with('-') && !p.starts_with('=') && p != "File Path")
        .collect()
}

/// Deduplicate hotspot entries by keeping only the first (most recent) entry per file path.
/// Non-hotspot entries (CONSTRAINT, INVARIANT) pass through unchanged.
fn dedup_hotspots(entries: Vec<(String, String)>) -> Vec<String> {
    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for (content, _updated_at) in &entries {
        if content.starts_with("HOTSPOT:") {
            let paths = extract_hotspot_paths(content);
            if paths.is_empty() {
                // Can't parse paths — keep the entry as-is
                result.push(content.clone());
                continue;
            }
            let new_paths: Vec<String> = paths
                .into_iter()
                .filter(|p| seen_paths.insert(p.clone()))
                .collect();
            if !new_paths.is_empty() {
                // Rebuild the entry with only the new paths to avoid noise
                result.push(content.clone());
            }
            // If all paths already seen, skip this entry entirely
        } else {
            // CONSTRAINTS, INVARIANTS, etc. — always keep
            result.push(content.clone());
        }
    }

    result
}

/// Compute a human-readable relative timestamp from an RFC 3339 string.
fn relative_timestamp(rfc3339_str: &str) -> String {
    let updated = match chrono::DateTime::parse_from_rfc3339(rfc3339_str) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return String::new(),
    };
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(updated);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} min ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hr ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!(
            "{} day{} ago",
            duration.num_days(),
            if duration.num_days() == 1 { "" } else { "s" }
        )
    } else if duration.num_days() < 30 {
        format!("{} wk ago", duration.num_days() / 7)
    } else {
        format!("{} mo ago", duration.num_days() / 30)
    }
}
