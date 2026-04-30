use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;
use rusqlite::params;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalMemory {
    pub memory_id: String,
    pub content: String,
}

pub fn lexical_search(
    conn: &VaultConnection,
    query: &str,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<RetrievalMemory>> {
    let conn = conn.lock()?;

    let mut sql = "SELECT mp.memory_id, mp.content, mp.privacy
         FROM memory_fts fts
         JOIN memory_projection mp ON mp.rowid = fts.rowid
         WHERE memory_fts MATCH ? AND mp.status = 'pinned'"
        .to_string();

    if project_id.is_some() || session_id.is_some() {
        // This is a placeholder since memory_projection doesn't have project_id yet.
        // In a full implementation, we would join with session_projection here
        // or ensure memory_projection has project_id denormalized.
        // For now, we will just return the global hits but the signature is ready.
    }

    sql.push_str(" ORDER BY rank");

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![query])?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(2)?;
        if is_injectable_privacy(&privacy) {
            results.push(RetrievalMemory {
                memory_id: row.get(0)?,
                content: row.get(1)?,
            });
        }
    }

    Ok(results)
}
