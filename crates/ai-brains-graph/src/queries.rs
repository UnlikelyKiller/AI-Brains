use crate::errors::{GraphError, Result};
use crate::ladybug::LadybugVault;
use lbug::Value;
use std::collections::HashMap;

pub struct GraphSearch<'a> {
    vault: &'a LadybugVault,
}

impl<'a> GraphSearch<'a> {
    pub fn new(vault: &'a LadybugVault) -> Self {
        Self { vault }
    }

    pub fn get_related_memories(&self, session_id: &str) -> Result<Vec<String>> {
        let conn = self.vault.connection()?;
        let mut params = HashMap::new();
        params.insert(
            "session_id".to_string(),
            Value::String(session_id.to_string()),
        );

        let mut result = conn.execute(
            "MATCH (s:Session {id: $session_id})<-[:IN_SESSION]-(t:Turn)-[:RECALLS|SOURCE_FOR]->(m:Memory) \
             WHERE m.forgotten IS NULL OR m.forgotten = false \
             RETURN m.id",
            params
        ).map_err(|e| GraphError::DbError(e.to_string()))?;

        let mut memories = Vec::new();
        while result.has_next() {
            let row = result
                .get_next()
                .map_err(|e| GraphError::DbError(e.to_string()))?;
            if let Some(id) = row.get_column(0).map(|v| v.to_string()) {
                memories.push(id);
            }
        }
        Ok(memories)
    }
}
