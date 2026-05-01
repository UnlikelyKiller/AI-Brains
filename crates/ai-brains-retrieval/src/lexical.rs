use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;
use rusqlite::params_from_iter;

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
         LEFT JOIN session_projection sp ON mp.session_id = sp.session_id
         WHERE memory_fts MATCH ? AND mp.status = 'pinned'"
        .to_string();

    let mut params_vec: Vec<rusqlite::types::Value> = vec![query.to_string().into()];

    if let Some(sid) = session_id {
        sql.push_str(" AND mp.session_id = ?");
        params_vec.push(sid.to_string().into());
    }

    if let Some(pid) = project_id {
        sql.push_str(" AND sp.project_id = ?");
        params_vec.push(pid.to_string().into());
    }

    sql.push_str(" ORDER BY rank");

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params_vec))?;
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
