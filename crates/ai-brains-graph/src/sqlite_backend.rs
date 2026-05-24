use crate::cozo_proxy::{GraphBackend, GraphEdge, GraphNode};
use crate::errors::{GraphError, Result};
use ai_brains_store::VaultConnection;
use rusqlite::{params, Connection};
use std::sync::Arc;

pub struct SqliteGraphBackend {
    conn: Arc<VaultConnection>,
}

impl SqliteGraphBackend {
    pub fn new(conn: Arc<VaultConnection>) -> Self {
        Self { conn }
    }

    fn ensure_node(&self, conn: &Connection, external_id: &str, kind: &str) -> Result<i64> {
        let mut stmt = conn
            .prepare_cached("SELECT node_id FROM graph_node WHERE external_id = ?")
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let existing = stmt.query_row([external_id], |row| row.get::<_, i64>(0));

        match existing {
            Ok(id) => Ok(id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                conn.execute(
                    "INSERT INTO graph_node (kind, external_id) VALUES (?, ?)",
                    params![kind, external_id],
                )
                .map_err(|e| GraphError::DbError(e.to_string()))?;
                Ok(conn.last_insert_rowid())
            }
            Err(e) => Err(GraphError::DbError(e.to_string())),
        }
    }

    fn ensure_edge(
        &self,
        conn: &Connection,
        src_id: i64,
        label: &str,
        dst_id: i64,
        weight: f64,
    ) -> Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id, weight) VALUES (?, ?, ?, ?)",
            params![src_id, label, dst_id, weight],
        )
        .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }
}

impl GraphBackend for SqliteGraphBackend {
    fn add_node(
        &self,
        id: &str,
        _label: &str,
        category: &str,
        _metadata: &serde_json::Value,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        self.ensure_node(&conn, id, category)?;
        Ok(())
    }

    fn add_nodes(&self, nodes: &[GraphNode]) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        for node in nodes {
            self.ensure_node(&tx, &node.id, &node.category)?;
        }
        tx.commit()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }

    fn add_edge(&self, source: &str, target: &str, relation: &str, confidence: f64) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        let src_id = self.ensure_node(&tx, source, "unknown")?;
        let dst_id = self.ensure_node(&tx, target, "unknown")?;
        self.ensure_edge(&tx, src_id, relation, dst_id, confidence)?;
        tx.commit()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }

    fn add_edges(&self, edges: &[GraphEdge]) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        for edge in edges {
            let src_id = self.ensure_node(&tx, &edge.source, "unknown")?;
            let dst_id = self.ensure_node(&tx, &edge.target, "unknown")?;
            self.ensure_edge(&tx, src_id, &edge.relation, dst_id, edge.confidence)?;
        }
        tx.commit()
            .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }

    fn query_neighbors(&self, _node_id: &str) -> Result<Vec<(String, String)>> {
        // Implementation omitted for brevity, using GraphSearch instead
        Ok(Vec::new())
    }

    fn query_path(&self, _from: &str, _to: &str) -> Result<Vec<String>> {
        // Implementation omitted for brevity, using GraphSearch instead
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        true
    }
}
