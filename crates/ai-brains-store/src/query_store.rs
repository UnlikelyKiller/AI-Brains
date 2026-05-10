use crate::connection::VaultConnection;
use crate::errors::Result;
use crate::QueryStore;
use ai_brains_core::ids::MemoryId;
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

        // Update last_accessed_at
        conn.execute(
            "UPDATE turn_projection SET last_accessed_at = ? WHERE session_id = ?",
            [chrono::Utc::now().to_rfc3339(), session_id.to_string()],
        )?;

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

    fn get_session_status(&self, session_id: &ai_brains_core::ids::SessionId) -> Result<Option<String>> {
        use rusqlite::{params, OptionalExtension};
        let conn = self.lock()?;
        let mut stmt = conn.prepare("SELECT status FROM session_projection WHERE session_id = ?")?;
        let status: Option<String> = stmt
            .query_row(params![session_id.to_string()], |row| row.get(0))
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

    fn update_memory_status(&self, memory_id: &MemoryId, status: &str) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE memory_projection SET status = ?, updated_at = ? WHERE memory_id = ?",
            [
                status,
                &chrono::Utc::now().to_rfc3339(),
                &memory_id.to_string(),
            ],
        )?;
        Ok(())
    }
}
