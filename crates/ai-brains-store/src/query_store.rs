use crate::connection::VaultConnection;
use crate::errors::Result;
use crate::QueryStore;
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use rusqlite::{params, OptionalExtension};
use std::str::FromStr;

impl QueryStore for VaultConnection {
    fn get_unsummarized_sessions(&self) -> Result<Vec<String>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT session_id FROM session_projection 
             WHERE status = 'completed' AND summary_memory_id IS NULL",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_session_turns(&self, session_id: &str) -> Result<Vec<(String, String)>> {
        let conn = self.lock()?;

        let mut stmt = conn.prepare(
            "SELECT role, content FROM turn_projection
             WHERE session_id = ?
             ORDER BY occurred_at ASC",
        )?;
        let rows = stmt.query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_session_status(&self, session_id: &SessionId) -> Result<Option<String>> {
        let conn = self.lock()?;
        let mut stmt =
            conn.prepare("SELECT status FROM session_projection WHERE session_id = ?")?;
        let status: Option<String> = stmt
            .query_row([session_id.to_string()], |row| row.get(0))
            .optional()?;
        Ok(status)
    }

    fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<(MemoryId, String)>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT f.memory_id, f.content 
             FROM memory_fts f
             JOIN memory_projection p ON f.memory_id = p.memory_id
             WHERE f.content MATCH ? AND p.status != 'forgotten'
             LIMIT ?",
        )?;
        let rows = stmt.query_map([query, &limit.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id_str, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id_str, content) = row?;
            let id = MemoryId::from_str(&id_str)
                .map_err(|e| crate::StoreError::EventReadFailed(e.to_string()))?;
            results.push((id, content));
        }
        Ok(results)
    }

    fn get_memories_by_level(&self, level: u32) -> Result<Vec<(MemoryId, String)>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT memory_id, content FROM memory_projection 
             WHERE level = ? AND status = 'pinned'",
        )?;
        let rows = stmt.query_map([level], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id_str, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id_str, content) = row?;
            let id = MemoryId::from_str(&id_str)
                .map_err(|e| crate::StoreError::EventReadFailed(e.to_string()))?;
            results.push((id, content));
        }
        Ok(results)
    }

    fn delete_old_turns(&self, cutoff: chrono::DateTime<chrono::Utc>) -> Result<usize> {
        let conn = self.lock()?;
        let count = conn.execute(
            "DELETE FROM turn_projection WHERE last_accessed_at < ?",
            [cutoff.to_rfc3339()],
        )?;
        Ok(count)
    }

    fn list_forgotten_memories(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<(String, String)>> {
        let conn = self.lock()?;
        let (sql, params): (String, Vec<String>) = if let Some(pid) = project_id {
            let pid_str = pid.to_string();
            (
                "SELECT mp.memory_id, mp.content FROM memory_projection mp \
                 LEFT JOIN session_projection sp ON mp.session_id = sp.session_id \
                 WHERE mp.status = 'forgotten' AND (sp.project_id = ? OR mp.project_id = ?) \
                 ORDER BY mp.updated_at DESC"
                    .into(),
                vec![pid_str.clone(), pid_str],
            )
        } else {
            (
                "SELECT memory_id, content FROM memory_projection \
                 WHERE status = 'forgotten' ORDER BY updated_at DESC"
                    .into(),
                vec![],
            )
        };

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn resolve_project_id_from_alias(&self, alias: &str) -> Result<Option<ProjectId>> {
        let conn = self.lock()?;
        let res: Option<String> = conn
            .query_row(
                "SELECT project_id FROM project_alias_projection WHERE alias = ?",
                [alias],
                |row| row.get(0),
            )
            .optional()?;

        match res {
            Some(s) => Ok(Some(ProjectId::from_str(&s).map_err(|e| {
                crate::errors::StoreError::EventReadFailed(e.to_string())
            })?)),
            None => Ok(None),
        }
    }

    fn get_max_turn_index(&self, session_id: &SessionId) -> Result<Option<i32>> {
        let conn = self.lock()?;
        let res: Option<i32> = conn
            .query_row(
                "SELECT MAX(turn_index) FROM turn_projection WHERE session_id = ?",
                [session_id.to_string()],
                |row| row.get::<_, Option<i32>>(0),
            )
            .optional()?
            .flatten();
        Ok(res)
    }

    fn get_sync_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare("SELECT value FROM sync_state WHERE key = ?")?;
        let res: Option<String> = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(res)
    }

    fn get_last_nightly_run(&self) -> Result<Option<String>> {
        self.get_sync_state("last_nightly_run")
    }
}
