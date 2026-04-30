use crate::errors::{GraphError, Result};
use crate::vault::GraphVault;
use ai_brains_events::{Envelope, Payload};
use rusqlite::{params, Connection};

pub struct GraphProjector<'a> {
    vault: &'a GraphVault,
}

impl<'a> GraphProjector<'a> {
    pub fn new(vault: &'a GraphVault) -> Self {
        Self { vault }
    }

    pub fn apply(&self, envelope: &Envelope) -> Result<()> {
        let conn = self
            .vault
            .connection()
            .lock()
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        match &envelope.payload {
            Payload::ProjectRegistered(p) => {
                let project_id = self.ensure_node(&conn, &p.project_id.to_string(), "project")?;
                tracing::debug!(project_id, "Project node ensured");
            }
            Payload::SessionStarted(s) => {
                let session_node_id =
                    self.ensure_node(&conn, &s.session_id.to_string(), "session")?;
                let project_node_id =
                    self.ensure_node(&conn, &s.project_id.to_string(), "project")?;

                self.ensure_edge(&conn, session_node_id, "IN_PROJECT", project_node_id)?;
            }
            Payload::UserPromptRecorded(p) => {
                let session_node_id =
                    self.ensure_node(&conn, &p.session_id.to_string(), "session")?;

                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                hasher.write(p.session_id.to_string().as_bytes());
                hasher.write(p.content.as_bytes());
                hasher.write(envelope.occurred_at.to_string().as_bytes());
                let turn_external_id = format!("{:x}", hasher.finish());

                let turn_node_id = self.ensure_node(&conn, &turn_external_id, "turn")?;
                self.ensure_edge(&conn, turn_node_id, "IN_SESSION", session_node_id)?;
            }
            Payload::AssistantFinalRecorded(p) => {
                let session_node_id =
                    self.ensure_node(&conn, &p.session_id.to_string(), "session")?;

                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                hasher.write(p.session_id.to_string().as_bytes());
                hasher.write(p.content.as_bytes());
                hasher.write(envelope.occurred_at.to_string().as_bytes());
                let turn_external_id = format!("{:x}", hasher.finish());

                let turn_node_id = self.ensure_node(&conn, &turn_external_id, "turn")?;
                self.ensure_edge(&conn, turn_node_id, "IN_SESSION", session_node_id)?;
            }
            Payload::MemoryPinned(p) => {
                let memory_node_id = self.ensure_node(&conn, &p.memory_id.to_string(), "memory")?;
                tracing::debug!(memory_node_id, "Memory node (pinned) ensured");
            }
            Payload::SessionSummaryCreated(p) => {
                let memory_node_id = self.ensure_node(&conn, &p.memory_id.to_string(), "memory")?;
                tracing::debug!(memory_node_id, "Memory node (summary) ensured");
            }
            Payload::MemorySynthesized(p) => {
                let memory_node_id = self.ensure_node(&conn, &p.memory_id.to_string(), "memory")?;

                for source_id in &p.source_memory_ids {
                    // Source could be a memory or a turn (for initial synthesis)
                    // We need to figure out what kind of source it is.
                    // Usually RAPTOR level 0 is from turns, level > 0 is from memories.
                    let source_kind = if p.level == 0 { "turn" } else { "memory" };
                    let source_node_id =
                        self.ensure_node(&conn, &source_id.to_string(), source_kind)?;
                    self.ensure_edge(&conn, memory_node_id, "SYNTHESIZED_FROM", source_node_id)?;
                }
            }
            Payload::ConflictDetected(p) => {
                let conflict_node_id =
                    self.ensure_node(&conn, &p.conflict_id.to_string(), "conflict")?;
                for memory_id in &p.contradicted_memory_ids {
                    let memory_node_id =
                        self.ensure_node(&conn, &memory_id.to_string(), "memory")?;
                    self.ensure_edge(&conn, conflict_node_id, "CONFLICTS_WITH", memory_node_id)?;
                }
            }
            Payload::RecipePromoted(p) => {
                let recipe_node_id = self.ensure_node(&conn, &p.recipe_id.to_string(), "recipe")?;
                for session_id in &p.source_session_ids {
                    let session_node_id =
                        self.ensure_node(&conn, &session_id.to_string(), "session")?;
                    self.ensure_edge(&conn, session_node_id, "PART_OF_RECIPE", recipe_node_id)?;
                }
            }
            Payload::MemoryForgotten(_) => {
                // In relational graph, we can just mark the node or let the query filter it.
                // For now, we don't have a 'forgotten' column in graph_node,
                // but the ADR says 'SQL and Graph search queries updated to exclude forgotten memories'.
                // I'll add a 'forgotten' column if needed, or just handle it in the queries by joining to memory_projection.
            }
            _ => {}
        }
        Ok(())
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

    fn ensure_edge(&self, conn: &Connection, src_id: i64, label: &str, dst_id: i64) -> Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id) VALUES (?, ?, ?)",
            params![src_id, label, dst_id],
        )
        .map_err(|e| GraphError::DbError(e.to_string()))?;
        Ok(())
    }
}
