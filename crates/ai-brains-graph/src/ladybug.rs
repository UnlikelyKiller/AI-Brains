use crate::errors::{GraphError, Result};
#[cfg(feature = "ladybug")]
use lbug::{Connection, Database, SystemConfig};
#[cfg(not(feature = "ladybug"))]
use std::collections::{HashMap, HashSet};
use std::path::Path;
#[cfg(not(feature = "ladybug"))]
use std::sync::{Arc, Mutex};

#[cfg(feature = "ladybug")]
pub use lbug::Value;

#[cfg(feature = "ladybug")]
pub struct LadybugVault {
    db: Database,
}

#[cfg(feature = "ladybug")]
impl LadybugVault {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db = Database::new(
            path.as_ref().to_string_lossy().to_string(),
            SystemConfig::default(),
        )
        .map_err(|e| GraphError::DbError(e.to_string()))?;

        let vault = Self { db };
        vault.initialize_schema()?;
        Ok(vault)
    }

    pub fn connection(&self) -> Result<Connection> {
        Connection::new(&self.db).map_err(|e| GraphError::DbError(e.to_string()))
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection()?;

        let tables_result = conn
            .query("CALL show_tables() RETURN name")
            .map_err(|e| GraphError::DbError(e.to_string()))?;

        let existing_tables: Vec<String> = tables_result
            .map(|row| row.get_column(0).map(|v| v.to_string()).unwrap_or_default())
            .collect();

        for (name, query) in crate::schema::TABLES {
            if !existing_tables.contains(&name.to_string()) {
                conn.query(query)
                    .map_err(|e| GraphError::SchemaError(e.to_string()))?;
            }
        }

        Ok(())
    }
}

#[cfg(not(feature = "ladybug"))]
#[derive(Default)]
struct GraphState {
    projects: HashMap<String, String>,
    sessions: HashSet<String>,
    session_projects: HashSet<(String, String)>,
    turns: HashSet<String>,
    turn_sessions: HashSet<(String, String)>,
    memories: HashSet<String>,
    forgotten_memories: HashSet<String>,
    synthesized_from: HashSet<(String, String)>,
    conflicts: HashSet<String>,
    conflict_memories: HashSet<(String, String)>,
    recipes: HashSet<String>,
    session_recipes: HashSet<(String, String)>,
}

#[cfg(not(feature = "ladybug"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int64(i64),
    Bool(bool),
}

#[cfg(not(feature = "ladybug"))]
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => write!(f, "{value}"),
            Self::Int64(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
        }
    }
}

#[cfg(not(feature = "ladybug"))]
pub struct LadybugVault {
    state: Arc<Mutex<GraphState>>,
}

#[cfg(not(feature = "ladybug"))]
impl LadybugVault {
    pub fn open(_path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(GraphState::default())),
        })
    }

    pub fn connection(&self) -> Result<Connection> {
        Ok(Connection {
            state: self.state.clone(),
        })
    }
}

#[cfg(not(feature = "ladybug"))]
pub struct Connection {
    state: Arc<Mutex<GraphState>>,
}

#[cfg(not(feature = "ladybug"))]
impl Connection {
    pub fn query(&self, query: &str) -> Result<QueryResult> {
        if query.contains("CALL show_tables()") {
            return Ok(QueryResult::new(Vec::new()));
        }

        if query.contains("MATCH (p:Project") && query.contains("RETURN p.name") {
            let project_id = extract_quoted_id(query).ok_or_else(|| {
                GraphError::DbError("project query missing id literal".to_string())
            })?;
            let state = self.lock_state()?;
            let rows = state
                .projects
                .get(&project_id)
                .map(|name| vec![vec![Value::String(name.clone())]])
                .unwrap_or_default();
            return Ok(QueryResult::new(rows));
        }

        if query.contains("MATCH (s:Session")
            && query.contains("IN_PROJECT")
            && query.contains("RETURN count(*)")
        {
            let ids = extract_all_quoted_ids(query);
            if ids.len() < 2 {
                return Err(GraphError::DbError(
                    "session/project query missing id literals".to_string(),
                ));
            }
            let state = self.lock_state()?;
            let count = i64::from(
                state
                    .session_projects
                    .contains(&(ids[0].clone(), ids[1].clone())),
            );
            return Ok(QueryResult::new(vec![vec![Value::Int64(count)]]));
        }

        Ok(QueryResult::new(Vec::new()))
    }

    pub fn execute(&self, query: &str, params: HashMap<String, Value>) -> Result<QueryResult> {
        if query.contains("RETURN m.id") {
            return self.related_memories(params);
        }

        let mut state = self.lock_state()?;
        if query.contains("SYNTHESIZED_FROM") {
            let id = string_param(&params, "id")?;
            let child_id = string_param(&params, "child_id")?;
            state.synthesized_from.insert((id, child_id));
        } else if query.contains("CONFLICTS_WITH") {
            let id = string_param(&params, "id")?;
            let memory_id = string_param(&params, "memory_id")?;
            state.conflict_memories.insert((id, memory_id));
        } else if query.contains("PART_OF_RECIPE") {
            let id = string_param(&params, "id")?;
            let session_id = string_param(&params, "session_id")?;
            state.session_recipes.insert((session_id, id));
        } else if query.contains("forgotten = true") {
            let id = string_param(&params, "id")?;
            state.forgotten_memories.insert(id);
        } else if query.contains("(s:Session") && query.contains("IN_PROJECT") {
            let id = string_param(&params, "id")?;
            let project_id = string_param(&params, "project_id")?;
            state.sessions.insert(id.clone());
            state.session_projects.insert((id, project_id));
        } else if query.contains("(t:Turn") && query.contains("IN_SESSION") {
            let id = string_param(&params, "id")?;
            let session_id = string_param(&params, "session_id")?;
            state.turns.insert(id.clone());
            state.turn_sessions.insert((id, session_id));
        } else if query.contains("(p:Project") {
            let id = string_param(&params, "id")?;
            let name = string_param(&params, "name").unwrap_or_else(|_| id.clone());
            state.projects.insert(id, name);
        } else if query.contains("(m:Memory") {
            let id = string_param(&params, "id")?;
            state.memories.insert(id);
        } else if query.contains("(c:Conflict") {
            let id = string_param(&params, "id")?;
            state.conflicts.insert(id);
        } else if query.contains("(r:Recipe") {
            let id = string_param(&params, "id")?;
            state.recipes.insert(id);
        }

        Ok(QueryResult::new(Vec::new()))
    }

    fn related_memories(&self, params: HashMap<String, Value>) -> Result<QueryResult> {
        let session_id = string_param(&params, "session_id")?;
        let state = self.lock_state()?;
        let turn_ids = state
            .turn_sessions
            .iter()
            .filter_map(|(turn_id, related_session_id)| {
                (related_session_id == &session_id).then_some(turn_id)
            })
            .collect::<HashSet<_>>();

        let rows = state
            .synthesized_from
            .iter()
            .filter(|(memory_id, source_id)| {
                turn_ids.contains(source_id) && !state.forgotten_memories.contains(memory_id)
            })
            .map(|(memory_id, _)| vec![Value::String(memory_id.clone())])
            .collect::<Vec<_>>();

        Ok(QueryResult::new(rows))
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, GraphState>> {
        self.state
            .lock()
            .map_err(|e| GraphError::DbError(format!("graph state lock poisoned: {e}")))
    }
}

#[cfg(not(feature = "ladybug"))]
pub struct QueryResult {
    rows: Vec<Vec<Value>>,
    index: usize,
}

#[cfg(not(feature = "ladybug"))]
impl QueryResult {
    fn new(rows: Vec<Vec<Value>>) -> Self {
        Self { rows, index: 0 }
    }

    pub fn has_next(&self) -> bool {
        self.index < self.rows.len()
    }

    pub fn get_next(&mut self) -> Result<QueryRow> {
        let row = self
            .rows
            .get(self.index)
            .cloned()
            .ok_or_else(|| GraphError::DbError("query result exhausted".to_string()))?;
        self.index += 1;
        Ok(QueryRow { values: row })
    }
}

#[cfg(not(feature = "ladybug"))]
pub struct QueryRow {
    values: Vec<Value>,
}

#[cfg(not(feature = "ladybug"))]
impl QueryRow {
    pub fn get_column(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

#[cfg(not(feature = "ladybug"))]
fn string_param(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    match params.get(name) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(value) => Err(GraphError::DbError(format!(
            "expected string parameter {name}, got {value:?}"
        ))),
        None => Err(GraphError::DbError(format!("missing parameter {name}"))),
    }
}

#[cfg(not(feature = "ladybug"))]
fn extract_quoted_id(query: &str) -> Option<String> {
    extract_all_quoted_ids(query).into_iter().next()
}

#[cfg(not(feature = "ladybug"))]
fn extract_all_quoted_ids(query: &str) -> Vec<String> {
    query
        .split("id: '")
        .skip(1)
        .filter_map(|part| part.split('\'').next().map(ToString::to_string))
        .collect()
}
