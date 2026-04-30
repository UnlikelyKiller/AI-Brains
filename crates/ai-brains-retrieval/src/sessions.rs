use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContext {
    pub session_id: String,
    pub turns: Vec<SessionTurn>,
}

pub fn active_sessions(
    conn: &VaultConnection,
    project_id: Option<ai_brains_core::ids::ProjectId>,
) -> Result<Vec<SessionContext>> {
    let conn = conn.lock()?;

    let mut sql = "SELECT session_id, privacy
         FROM session_projection
         WHERE status = 'active'"
        .to_string();

    if let Some(pid) = project_id {
        sql.push_str(&format!(" AND project_id = '{}'", pid));
    }

    sql.push_str(" ORDER BY updated_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    let mut active = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(1)?;
        if is_injectable_privacy(&privacy) {
            let session_id: String = row.get(0)?;

            // Fetch last 5 turns for this session
            let mut turn_stmt = conn.prepare(
                "SELECT role, content
                 FROM turn_projection
                 WHERE session_id = ?
                 ORDER BY turn_index DESC
                 LIMIT 5",
            )?;
            let mut turn_rows = turn_stmt.query([&session_id])?;
            let mut turns = Vec::new();
            while let Some(turn_row) = turn_rows.next()? {
                turns.push(SessionTurn {
                    role: turn_row.get(0)?,
                    content: turn_row.get(1)?,
                });
            }
            // Reverse to get chronological order
            turns.reverse();

            active.push(SessionContext { session_id, turns });
        }
    }

    Ok(active)
}
