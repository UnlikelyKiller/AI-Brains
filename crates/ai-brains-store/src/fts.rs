use crate::errors::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub memory_id: String,
    pub project_id: Option<String>,
    pub content_markdown: String,
}

pub struct FtsSearch<'a> {
    conn: &'a Connection,
}

impl<'a> FtsSearch<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn search(&self, query: &str, project_id: Option<Uuid>) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let (sql, params_vec) = if let Some(pid) = project_id {
            (
                "SELECT f.memory_id, p.project_id, f.content AS content_markdown
                 FROM memory_fts f
                 JOIN memory_projection p ON f.memory_id = p.memory_id
                 WHERE memory_fts MATCH ? AND p.project_id = ?
                 ORDER BY rank",
                params![query, pid.to_string()],
            )
        } else {
            (
                "SELECT f.memory_id, p.project_id, f.content AS content_markdown
                 FROM memory_fts f
                 JOIN memory_projection p ON f.memory_id = p.memory_id
                 WHERE memory_fts MATCH ?
                 ORDER BY rank",
                params![query],
            )
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query(params_vec)?;

        while let Some(row) = rows.next()? {
            let memory_id: String = row.get(0)?;
            let project_id: Option<String> = row.get(1)?;
            let content_markdown: String = row.get(2)?;

            results.push(SearchResult {
                memory_id,
                project_id,
                content_markdown,
            });
        }

        Ok(results)
    }
}

/// Unscoped search for backward compatibility in tests.
/// Production code should use FtsSearch directly with a ProjectId.
pub fn search_memory(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let fts = FtsSearch::new(conn);
    fts.search(query, None)
}
