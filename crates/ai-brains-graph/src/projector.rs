use crate::errors::{GraphError, Result};
use crate::ladybug::{LadybugVault, Value};
use ai_brains_events::{Envelope, Payload};
use std::collections::HashMap;

pub struct GraphProjector<'a> {
    vault: &'a LadybugVault,
}

impl<'a> GraphProjector<'a> {
    pub fn new(vault: &'a LadybugVault) -> Self {
        Self { vault }
    }

    pub fn apply(&self, envelope: &Envelope) -> Result<()> {
        let conn = self.vault.connection()?;

        match &envelope.payload {
            Payload::ProjectRegistered(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.project_id.to_string()));
                params.insert("name".to_string(), Value::String(p.name.clone()));
                conn.execute(
                    "MERGE (p:Project {id: $id}) ON CREATE SET p.name = $name",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            Payload::SessionStarted(s) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(s.session_id.to_string()));
                params.insert(
                    "started_at".to_string(),
                    Value::String(envelope.occurred_at.to_string()),
                );
                params.insert(
                    "project_id".to_string(),
                    Value::String(s.project_id.to_string()),
                );
                params.insert("harness".to_string(), Value::String("claude".to_string())); // Default for now
                conn.execute(
                    "MERGE (s:Session {id: $id}) \
                     ON CREATE SET s.started_at = $started_at, s.harness = $harness \
                     WITH s \
                     MATCH (p:Project {id: $project_id}) \
                     MERGE (s)-[:IN_PROJECT]->(p)",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            Payload::UserPromptRecorded(p) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                hasher.write(p.session_id.to_string().as_bytes());
                hasher.write(p.content.as_bytes());
                hasher.write(envelope.occurred_at.to_string().as_bytes());
                let turn_id = format!("{:x}", hasher.finish());

                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(turn_id));
                params.insert(
                    "session_id".to_string(),
                    Value::String(p.session_id.to_string()),
                );
                conn.execute(
                    "MERGE (t:Turn {id: $id}) \
                     ON CREATE SET t.role = 'user' \
                     WITH t \
                     MATCH (s:Session {id: $session_id}) \
                     MERGE (t)-[:IN_SESSION]->(s)",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            Payload::AssistantFinalRecorded(p) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                use std::hash::Hasher;
                hasher.write(p.session_id.to_string().as_bytes());
                hasher.write(p.content.as_bytes());
                hasher.write(envelope.occurred_at.to_string().as_bytes());
                let turn_id = format!("{:x}", hasher.finish());

                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(turn_id));
                params.insert(
                    "session_id".to_string(),
                    Value::String(p.session_id.to_string()),
                );
                conn.execute(
                    "MERGE (t:Turn {id: $id}) \
                     ON CREATE SET t.role = 'assistant' \
                     WITH t \
                     MATCH (s:Session {id: $session_id}) \
                     MERGE (t)-[:IN_SESSION]->(s)",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            Payload::MemoryPinned(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.memory_id.to_string()));
                params.insert("kind".to_string(), Value::String("pinned".to_string()));
                conn.execute(
                    "MERGE (m:Memory {id: $id}) ON CREATE SET m.kind = $kind, m.level = 0",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            Payload::SessionSummaryCreated(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.memory_id.to_string()));
                params.insert("kind".to_string(), Value::String("summary".to_string()));
                conn.execute(
                    "MERGE (m:Memory {id: $id}) ON CREATE SET m.kind = $kind, m.level = 0",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            Payload::MemorySynthesized(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.memory_id.to_string()));
                params.insert("kind".to_string(), Value::String("synthesized".to_string()));
                params.insert("level".to_string(), Value::Int64(p.level as i64));
                conn.execute(
                    "MERGE (m:Memory {id: $id}) ON CREATE SET m.kind = $kind, m.level = $level",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;

                for child_id in &p.source_memory_ids {
                    let mut rel_params = HashMap::new();
                    rel_params.insert("id".to_string(), Value::String(p.memory_id.to_string()));
                    rel_params.insert("child_id".to_string(), Value::String(child_id.to_string()));
                    conn.execute(
                        "MATCH (p:Memory {id: $id}), (c:Memory {id: $child_id}) \
                         MERGE (p)-[:SYNTHESIZED_FROM]->(c)",
                        rel_params,
                    )
                    .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
                }
            }
            Payload::ConflictDetected(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.conflict_id.to_string()));
                params.insert(
                    "explanation".to_string(),
                    Value::String(p.explanation.clone()),
                );
                conn.execute(
                    "MERGE (c:Conflict {id: $id}) ON CREATE SET c.explanation = $explanation",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;

                for memory_id in &p.contradicted_memory_ids {
                    let mut rel_params = HashMap::new();
                    rel_params.insert("id".to_string(), Value::String(p.conflict_id.to_string()));
                    rel_params.insert(
                        "memory_id".to_string(),
                        Value::String(memory_id.to_string()),
                    );
                    conn.execute(
                        "MATCH (c:Conflict {id: $id}), (m:Memory {id: $memory_id}) \
                         MERGE (c)-[:CONFLICTS_WITH]->(m)",
                        rel_params,
                    )
                    .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
                }
            }
            Payload::RecipePromoted(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.recipe_id.to_string()));
                params.insert("name".to_string(), Value::String(p.name.clone()));
                conn.execute(
                    "MERGE (r:Recipe {id: $id}) ON CREATE SET r.name = $name",
                    params,
                )
                .map_err(|e| GraphError::ProjectionError(e.to_string()))?;

                for session_id in &p.source_session_ids {
                    let mut rel_params = HashMap::new();
                    rel_params.insert("id".to_string(), Value::String(p.recipe_id.to_string()));
                    rel_params.insert(
                        "session_id".to_string(),
                        Value::String(session_id.to_string()),
                    );
                    conn.execute(
                        "MATCH (s:Session {id: $session_id}), (r:Recipe {id: $id}) \
                         MERGE (s)-[:PART_OF_RECIPE]->(r)",
                        rel_params,
                    )
                    .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
                }
            }
            Payload::MemoryForgotten(p) => {
                let mut params = HashMap::new();
                params.insert("id".to_string(), Value::String(p.memory_id.to_string()));
                conn.execute("MATCH (m:Memory {id: $id}) SET m.forgotten = true", params)
                    .map_err(|e| GraphError::ProjectionError(e.to_string()))?;
            }
            _ => {}
        }
        Ok(())
    }
}
