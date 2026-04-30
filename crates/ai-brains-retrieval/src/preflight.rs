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

    let mut sql = "SELECT content, privacy
         FROM memory_projection
         WHERE status = 'pinned'"
        .to_string();

    // Placeholder for memory project filtering

    sql.push_str(" ORDER BY updated_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    let mut collected = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(1)?;
        if !is_injectable_privacy(&privacy) {
            continue;
        }

        let content: String = row.get(0)?;
        let candidate = if collected.is_empty() {
            content.clone()
        } else {
            format!("{}\n\n{}", collected.join("\n\n"), content)
        };

        if word_count(&candidate) > max_words {
            break;
        }
        collected.push(content);
    }

    let mut sections = Vec::new();
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
    if !collected.is_empty() {
        let mut index_lines = vec!["--- Memory Index (Briefing) ---".to_string()];
        for (i, content) in collected.iter().enumerate() {
            let first_line = content.lines().next().unwrap_or("Untitled Memory");
            let summary = if first_line.len() > 60 {
                format!("{}...", &first_line[..57])
            } else {
                first_line.to_string()
            };
            index_lines.push(format!("{}. {}", i + 1, summary));
        }
        sections.push(index_lines.join("\n"));

        sections.push(format!(
            "--- Detailed Memories ---\n\n{}",
            collected.join("\n\n")
        ));
    }

    let text = trim_to_word_budget(&sections.join("\n\n"), max_words);
    Ok(PreflightContext {
        word_count: word_count(&text),
        text,
    })
}
