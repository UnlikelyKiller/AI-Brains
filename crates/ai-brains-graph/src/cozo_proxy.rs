//! CozoDB proxy backend: translates AI-Brains graph operations into Datalog
//! statements and routes them through BridgeRecord IPC to ChangeGuard's CozoDB.
//!
//! Feature-gated: activates only when `.changeguard/` directory is present.
//! Falls back gracefully to the SQLite graph backend otherwise.

use crate::errors::{GraphError, Result};
use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_contracts::response::ApiResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A node in the AI-Brains knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub category: String,
    pub metadata: serde_json::Value,
}

/// An edge connecting two nodes in the AI-Brains knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: f64,
}

/// A path result from a reachability traversal query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
}

/// Parsed CozoDB response carrying named rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CozoNamedRows {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// GraphBackend trait
// ---------------------------------------------------------------------------

/// Abstraction over graph storage backends. Implementations route mutations
/// and queries to either a local SQLite store or a remote CozoDB instance
/// via the ChangeGuard bridge.
pub trait GraphBackend {
    /// Insert or update a node in the graph.
    fn add_node(
        &self,
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> Result<()>;

    /// Insert or update multiple nodes in a single batch.
    fn add_nodes(&self, nodes: &[GraphNode]) -> Result<()>;

    /// Insert or update an edge between two nodes.
    fn add_edge(&self, source: &str, target: &str, relation: &str, confidence: f64) -> Result<()>;

    /// Insert or update multiple edges in a single batch.
    fn add_edges(&self, edges: &[GraphEdge]) -> Result<()>;

    /// Query neighbors of a node, returning (target_id, relation) pairs.
    fn query_neighbors(&self, node_id: &str) -> Result<Vec<(String, String)>>;

    /// Find a path from `from` to `to`, returning the sequence of node IDs.
    fn query_path(&self, from: &str, to: &str) -> Result<Vec<String>>;

    /// Returns true when the backend is available and functional.
    fn is_available(&self) -> bool;
}

// ---------------------------------------------------------------------------
// CozoProxyBackend
// ---------------------------------------------------------------------------

/// Translates AI-Brains graph operations into CozoDB Datalog statements and
/// routes them through the ChangeGuard Bridge IPC (named pipe / CLI).
pub struct CozoProxyBackend {
    changeguard_available: bool,
}

impl CozoProxyBackend {
    pub fn new(working_dir: Option<PathBuf>) -> Self {
        let cwd = working_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let changeguard_dir = cwd.join(".changeguard");
        let dir_exists = changeguard_dir.exists() && changeguard_dir.is_dir();

        let cli_available = std::process::Command::new("changeguard")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let available = dir_exists && cli_available;

        tracing::info!(
            changeguard_dir=%changeguard_dir.display(),
            available,
            "CozoProxyBackend initialized"
        );

        Self {
            changeguard_available: available,
        }
    }

    /// Send a Datalog mutation (put) to ChangeGuard via bridge import.
    #[allow(clippy::disallowed_methods)]
    fn send_datalog_mutation(&self, datalog: &str, record_kind: &str) -> Result<()> {
        if !self.changeguard_available {
            return Err(GraphError::DbError(
                "ChangeGuard is not available; CozoProxyBackend cannot route mutations."
                    .to_string(),
            ));
        }

        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| GraphError::IoError(std::io::Error::other(e)))?;
        let temp_path = temp_file.path().to_path_buf();

        let timestamp = chrono::Utc::now();
        let record = BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp,
            parent_hash: None,
            project_id: "ChangeGuard".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: record_kind.to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "datalog": datalog,
            })),
            privacy: ai_brains_core::privacy::Privacy::LocalOnly,
        };

        let ndjson = serde_json::to_string(&record)
            .map_err(|e| GraphError::DbError(format!("Failed to serialize BridgeRecord: {}", e)))?;
        std::fs::write(&temp_path, ndjson.as_bytes()).map_err(GraphError::IoError)?;

        let output = std::process::Command::new("changeguard")
            .args([
                "bridge",
                "import",
                "--input",
                temp_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| {
                GraphError::DbError(format!("Failed to invoke changeguard bridge import: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if let Ok(api_res) = serde_json::from_str::<ApiResult<serde_json::Value>>(&stderr) {
                if let Some(err) = api_res.error {
                    return Err(GraphError::DbError(format!(
                        "ChangeGuard Error: {} ({})",
                        err.message, err.code
                    )));
                }
            }

            tracing::warn!("ChangeGuard bridge import failed: {}", stderr);
            return Err(GraphError::DbError(format!(
                "ChangeGuard bridge import rejected Datalog mutation: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Run a Datalog query through the bridge and parse NamedRows response.
    #[allow(clippy::disallowed_methods)]
    fn run_datalog_query(&self, datalog: &str) -> Result<CozoNamedRows> {
        if !self.changeguard_available {
            return Err(GraphError::DbError(
                "ChangeGuard is not available; CozoProxyBackend cannot run queries.".to_string(),
            ));
        }

        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| GraphError::IoError(std::io::Error::other(e)))?;
        let temp_path = temp_file.path().to_path_buf();

        let timestamp = chrono::Utc::now();
        let record = BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp,
            parent_hash: None,
            project_id: "ChangeGuard".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "datalog_query".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "datalog": datalog,
            })),
            privacy: ai_brains_core::privacy::Privacy::LocalOnly,
        };

        let ndjson = serde_json::to_string(&record).map_err(|e| {
            GraphError::DbError(format!("Failed to serialize query BridgeRecord: {}", e))
        })?;
        std::fs::write(&temp_path, ndjson.as_bytes()).map_err(GraphError::IoError)?;

        let out_file = tempfile::NamedTempFile::new()
            .map_err(|e| GraphError::IoError(std::io::Error::other(e)))?;
        let out_path = out_file.path().to_path_buf();

        let output = std::process::Command::new("changeguard")
            .args([
                "bridge",
                "export",
                "--out",
                out_path.to_str().unwrap_or(""),
                "--graph-query",
                temp_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| {
                GraphError::DbError(format!("Failed to invoke changeguard bridge export: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GraphError::DbError(format!(
                "ChangeGuard bridge export failed: {}",
                stderr
            )));
        }

        let content = std::fs::read_to_string(&out_path).map_err(GraphError::IoError)?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<BridgeRecord>(line) {
                if record.record_kind == "named_rows" {
                    if let Ok(rows) =
                        serde_json::from_value::<CozoNamedRows>(record.payload_value())
                    {
                        return Ok(rows);
                    }
                }
            }
        }

        Ok(CozoNamedRows {
            headers: Vec::new(),
            rows: Vec::new(),
        })
    }

    /// Translate `add_node` into a CozoDB `:put` Datalog statement.
    fn datalog_put_node(
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> String {
        let meta_str = serde_json::to_string(metadata).unwrap_or_else(|_| "null".to_string());
        format!(
            "?[id, label, category, metadata] <- [[\"{}\", \"{}\", \"{}\", {}]] :put node",
            escape_datalog_str(id),
            escape_datalog_str(label),
            escape_datalog_str(category),
            meta_str
        )
    }

    /// Translate `add_edge` into a CozoDB `:put` Datalog statement.
    fn datalog_put_edge(source: &str, target: &str, relation: &str, confidence: f64) -> String {
        format!(
            "?[source, target, relation, confidence] <- [[\"{}\", \"{}\", \"{}\", {}]] :put edge",
            escape_datalog_str(source),
            escape_datalog_str(target),
            escape_datalog_str(relation),
            confidence
        )
    }

    /// Translate `query_neighbors` into a CozoDB Datalog query.
    fn datalog_query_neighbors(node_id: &str) -> String {
        format!(
            "?[target, relation] := *edge{{source: \"{}\", target, relation}}",
            escape_datalog_str(node_id)
        )
    }

    /// Translate `query_path` into a CozoDB reachability traversal.
    fn datalog_query_path(from: &str, to: &str) -> String {
        format!(
            "?[path] := *reachable{{source: \"{}\", target: \"{}\", path}}",
            escape_datalog_str(from),
            escape_datalog_str(to)
        )
    }
}

// ---------------------------------------------------------------------------
// GraphBackend impl for CozoProxyBackend
// ---------------------------------------------------------------------------

impl GraphBackend for CozoProxyBackend {
    fn add_node(
        &self,
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        let node = GraphNode {
            id: id.to_string(),
            label: label.to_string(),
            category: category.to_string(),
            metadata: metadata.clone(),
        };
        self.add_nodes(&[node])
    }

    fn add_nodes(&self, nodes: &[GraphNode]) -> Result<()> {
        if nodes.is_empty() {
            return Ok(());
        }

        let mut datalog = "?[id, label, category, metadata] <- [".to_string();
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 {
                datalog.push_str(", ");
            }
            let meta_str =
                serde_json::to_string(&node.metadata).unwrap_or_else(|_| "null".to_string());
            datalog.push_str(&format!(
                "[\"{}\", \"{}\", \"{}\", {}]",
                escape_datalog_str(&node.id),
                escape_datalog_str(&node.label),
                escape_datalog_str(&node.category),
                meta_str
            ));
        }
        datalog.push_str("] :put node");

        tracing::debug!(count = nodes.len(), "CozoProxyBackend::add_nodes");
        self.send_datalog_mutation(&datalog, "datalog_put_node")
    }

    fn add_edge(&self, source: &str, target: &str, relation: &str, confidence: f64) -> Result<()> {
        let edge = GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            confidence,
        };
        self.add_edges(&[edge])
    }

    fn add_edges(&self, edges: &[GraphEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }

        let mut datalog = "?[source, target, relation, confidence] <- [".to_string();
        for (i, edge) in edges.iter().enumerate() {
            if i > 0 {
                datalog.push_str(", ");
            }
            datalog.push_str(&format!(
                "[\"{}\", \"{}\", \"{}\", {}]",
                escape_datalog_str(&edge.source),
                escape_datalog_str(&edge.target),
                escape_datalog_str(&edge.relation),
                edge.confidence
            ));
        }
        datalog.push_str("] :put edge");

        tracing::debug!(count = edges.len(), "CozoProxyBackend::add_edges");
        self.send_datalog_mutation(&datalog, "datalog_put_edge")
    }

    fn query_neighbors(&self, node_id: &str) -> Result<Vec<(String, String)>> {
        let datalog = Self::datalog_query_neighbors(node_id);
        tracing::debug!(node_id, "CozoProxyBackend::query_neighbors");
        let rows = self.run_datalog_query(&datalog)?;

        let mut results = Vec::new();
        for row in &rows.rows {
            if row.len() >= 2 {
                let target = row[0].as_str().unwrap_or("").to_string();
                let relation = row[1].as_str().unwrap_or("").to_string();
                if !target.is_empty() {
                    results.push((target, relation));
                }
            }
        }
        Ok(results)
    }

    fn query_path(&self, from: &str, to: &str) -> Result<Vec<String>> {
        let datalog = Self::datalog_query_path(from, to);
        tracing::debug!(from, to, "CozoProxyBackend::query_path");
        let rows = self.run_datalog_query(&datalog)?;

        let mut results = Vec::new();
        for row in &rows.rows {
            if let Some(val) = row.first() {
                if let Some(arr) = val.as_array() {
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            results.push(s.to_string());
                        }
                    }
                } else if let Some(s) = val.as_str() {
                    results.push(s.to_string());
                }
            }
        }
        Ok(results)
    }

    fn is_available(&self) -> bool {
        self.changeguard_available
    }
}

fn escape_datalog_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn datalog_put_node_produces_valid_syntax() {
        let metadata = serde_json::json!({"key": "value"});
        let datalog = CozoProxyBackend::datalog_put_node("node-1", "TestNode", "memory", &metadata);
        assert!(datalog.contains(":put node"));
    }

    #[test]
    fn datalog_put_edge_produces_valid_syntax() {
        let datalog = CozoProxyBackend::datalog_put_edge("src-1", "dst-1", "RELATES_TO", 0.95);
        assert!(datalog.contains(":put edge"));
    }

    #[test]
    fn escape_datalog_str_handles_special_chars() {
        let result = escape_datalog_str(r#"hello "world" \test"#);
        assert_eq!(result, r#"hello \"world\" \\test"#);
    }
}
