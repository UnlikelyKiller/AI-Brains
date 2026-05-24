use crate::cozo_proxy::{GraphBackend, GraphEdge, GraphNode};
use crate::errors::Result;
use ai_brains_events::{Envelope, Payload};

pub struct GraphProjector<'a> {
    backend: Box<dyn GraphBackend + Send + Sync + 'a>,
    node_buffer: Vec<GraphNode>,
    edge_buffer: Vec<GraphEdge>,
}

impl<'a> GraphProjector<'a> {
    pub fn new(backend: Box<dyn GraphBackend + Send + Sync + 'a>) -> Self {
        Self {
            backend,
            node_buffer: Vec::new(),
            edge_buffer: Vec::new(),
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        if !self.node_buffer.is_empty() {
            self.backend.add_nodes(&self.node_buffer)?;
            self.node_buffer.clear();
        }
        if !self.edge_buffer.is_empty() {
            self.backend.add_edges(&self.edge_buffer)?;
            self.edge_buffer.clear();
        }
        Ok(())
    }

    pub fn apply(&mut self, envelope: &Envelope) -> Result<()> {
        match &envelope.payload {
            Payload::ProjectRegistered(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.project_id.to_string(),
                    label: p.name.clone(),
                    category: "project".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
            Payload::SessionStarted(s) => {
                self.node_buffer.push(GraphNode {
                    id: s.session_id.to_string(),
                    label: "Session".to_string(),
                    category: "session".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: s.session_id.to_string(),
                    target: s.project_id.to_string(),
                    relation: "IN_PROJECT".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::UserPromptRecorded(p) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                hasher.write(p.session_id.to_string().as_bytes());
                hasher.write(p.content.as_bytes());
                hasher.write(envelope.occurred_at.to_string().as_bytes());
                let turn_id = format!("{:x}", hasher.finish());

                self.node_buffer.push(GraphNode {
                    id: turn_id.clone(),
                    label: "Turn".to_string(),
                    category: "turn".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: turn_id,
                    target: p.session_id.to_string(),
                    relation: "IN_SESSION".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::AssistantFinalRecorded(p) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                hasher.write(p.session_id.to_string().as_bytes());
                hasher.write(p.content.as_bytes());
                hasher.write(envelope.occurred_at.to_string().as_bytes());
                let turn_id = format!("{:x}", hasher.finish());

                self.node_buffer.push(GraphNode {
                    id: turn_id.clone(),
                    label: "Turn".to_string(),
                    category: "turn".to_string(),
                    metadata: serde_json::json!({}),
                });
                self.edge_buffer.push(GraphEdge {
                    source: turn_id,
                    target: p.session_id.to_string(),
                    relation: "IN_SESSION".to_string(),
                    confidence: 1.0,
                });
            }
            Payload::MemoryPinned(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.memory_id.to_string(),
                    label: "Memory".to_string(),
                    category: "memory".to_string(),
                    metadata: serde_json::json!({"status": "pinned"}),
                });
            }
            Payload::SessionSummaryCreated(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.memory_id.to_string(),
                    label: "Summary".to_string(),
                    category: "memory".to_string(),
                    metadata: serde_json::json!({"type": "summary"}),
                });
            }
            Payload::MemorySynthesized(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.memory_id.to_string(),
                    label: "Synthesized".to_string(),
                    category: "memory".to_string(),
                    metadata: serde_json::json!({"level": p.level}),
                });

                for source_id in &p.source_memory_ids {
                    let source_kind = if p.level == 0 { "turn" } else { "memory" };
                    self.node_buffer.push(GraphNode {
                        id: source_id.to_string(),
                        label: "Source".to_string(),
                        category: source_kind.to_string(),
                        metadata: serde_json::json!({}),
                    });
                    self.edge_buffer.push(GraphEdge {
                        source: p.memory_id.to_string(),
                        target: source_id.to_string(),
                        relation: "SYNTHESIZED_FROM".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::ConflictDetected(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.conflict_id.to_string(),
                    label: "Conflict".to_string(),
                    category: "conflict".to_string(),
                    metadata: serde_json::json!({}),
                });
                for memory_id in &p.memory_ids {
                    self.edge_buffer.push(GraphEdge {
                        source: p.conflict_id.to_string(),
                        target: memory_id.to_string(),
                        relation: "CONFLICTS_WITH".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            Payload::RecipePromoted(p) => {
                self.node_buffer.push(GraphNode {
                    id: p.recipe_id.to_string(),
                    label: "Recipe".to_string(),
                    category: "recipe".to_string(),
                    metadata: serde_json::json!({}),
                });
                for memory_id in &p.source_memory_ids {
                    self.edge_buffer.push(GraphEdge {
                        source: memory_id.to_string(),
                        target: p.recipe_id.to_string(),
                        relation: "PART_OF_RECIPE".to_string(),
                        confidence: 1.0,
                    });
                }
            }
            _ => {}
        }

        // Auto-flush if buffer gets too large
        if self.node_buffer.len() >= 100 || self.edge_buffer.len() >= 100 {
            self.flush()?;
        }

        Ok(())
    }
}
