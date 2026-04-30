use crate::errors::{GraphError, Result};
use crate::vault::GraphVault;

pub struct GraphSearch<'a> {
    vault: &'a GraphVault,
}

impl<'a> GraphSearch<'a> {
    pub fn new(vault: &'a GraphVault) -> Self {
        Self { vault }
    }

    /// Finds all memories related to a session (Session <- Turn -> Memory)
    pub fn get_related_memories(&self, session_id: &str) -> Result<Vec<String>> {
        let conn = self
            .vault
            .connection()
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let mut stmt = conn.prepare_cached(
            "WITH RECURSIVE walk(depth, node_id) AS (
                SELECT 0, node_id FROM graph_node WHERE external_id = ? AND kind = 'session'
                UNION ALL
                -- Session <-[:IN_SESSION]- Turn
                SELECT 1, e.src_id
                FROM walk w
                JOIN graph_edge e ON w.depth = 0 AND e.dst_id = w.node_id AND e.label = 'IN_SESSION'
                UNION ALL
                -- Turn -[:RECALLS|SOURCE_FOR]-> Memory
                SELECT 2, e.dst_id
                FROM walk w
                JOIN graph_edge e ON w.depth = 1 AND e.src_id = w.node_id AND e.label IN ('RECALLS', 'SOURCE_FOR')
            )
            SELECT DISTINCT n.external_id
            FROM walk w
            JOIN graph_node n ON n.node_id = w.node_id
            WHERE w.depth = 2"
        ).map_err(|e| GraphError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map([session_id], |row| row.get::<_, String>(0))
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| GraphError::DbError(e.to_string()))?);
        }
        Ok(results)
    }

    /// Finds all synthesized memories from a root memory (Recursive SYNTHESIZED_FROM)
    pub fn get_synthesized_hierarchy(&self, root_memory_id: &str) -> Result<Vec<String>> {
        let conn = self
            .vault
            .connection()
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let mut stmt = conn
            .prepare_cached(
                "WITH RECURSIVE r(node_id, depth) AS (
                SELECT node_id, 0 FROM graph_node WHERE external_id = ? AND kind = 'memory'
                UNION ALL
                SELECT e.dst_id, r.depth + 1
                FROM r
                JOIN graph_edge e ON e.src_id = r.node_id AND e.label = 'SYNTHESIZED_FROM'
                WHERE r.depth < 10
            )
            SELECT DISTINCT n.external_id
            FROM r
            JOIN graph_node n ON n.node_id = r.node_id
            WHERE r.depth > 0",
            )
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let rows = stmt
            .query_map([root_memory_id], |row| row.get::<_, String>(0))
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| GraphError::DbError(e.to_string()))?);
        }
        Ok(results)
    }
}
