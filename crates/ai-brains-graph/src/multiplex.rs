use crate::cozo_proxy::{GraphBackend, GraphEdge, GraphNode};
use crate::errors::Result;

pub struct MultiplexGraphBackend {
    backends: Vec<Box<dyn GraphBackend + Send + Sync>>,
}

impl MultiplexGraphBackend {
    pub fn new(backends: Vec<Box<dyn GraphBackend + Send + Sync>>) -> Self {
        Self { backends }
    }
}

impl GraphBackend for MultiplexGraphBackend {
    fn add_node(
        &self,
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        for backend in &self.backends {
            if backend.is_available() {
                backend.add_node(id, label, category, metadata)?;
            }
        }
        Ok(())
    }

    fn add_nodes(&self, nodes: &[GraphNode]) -> Result<()> {
        for backend in &self.backends {
            if backend.is_available() {
                backend.add_nodes(nodes)?;
            }
        }
        Ok(())
    }

    fn add_edge(&self, source: &str, target: &str, relation: &str, confidence: f64) -> Result<()> {
        for backend in &self.backends {
            if backend.is_available() {
                backend.add_edge(source, target, relation, confidence)?;
            }
        }
        Ok(())
    }

    fn add_edges(&self, edges: &[GraphEdge]) -> Result<()> {
        for backend in &self.backends {
            if backend.is_available() {
                backend.add_edges(edges)?;
            }
        }
        Ok(())
    }

    fn query_neighbors(&self, node_id: &str) -> Result<Vec<(String, String)>> {
        // Return first available result
        for backend in &self.backends {
            if backend.is_available() {
                return backend.query_neighbors(node_id);
            }
        }
        Ok(Vec::new())
    }

    fn query_path(&self, from: &str, to: &str) -> Result<Vec<String>> {
        // Return first available result
        for backend in &self.backends {
            if backend.is_available() {
                return backend.query_path(from, to);
            }
        }
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        self.backends.iter().any(|b| b.is_available())
    }
}
