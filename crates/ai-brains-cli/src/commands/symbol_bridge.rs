use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    constructors::EventBuilder, Actor, AggregateType, EventKind, MemoryPinnedPayload, Payload,
};
use serde::Deserialize;
use uuid::Uuid;

use ai_brains_store::EventStore;

#[derive(Clone, Debug, Deserialize)]
struct SymbolRecord {
    file_path: String,
    qualified_name: String,
    #[allow(dead_code)]
    symbol_name: String,
    symbol_kind: String,
    line_start: i64,
    method: Option<String>,
    path_pattern: Option<String>,
}

/// Refresh ChangeGuard's symbol index, then ingest public symbols into AI-Brains
/// as MemoryPinned events. Non-fatal; any failure is logged and skipped.
pub fn ingest_symbols_from_changeguard(
    ctx: &AppContext,
    project_id: ProjectId,
) -> Result<usize, Box<dyn std::error::Error>> {
    refresh_changeguard_index();

    let project_root = std::env::current_dir().ok();
    let symbols = query_symbols_from_changeguard()?;
    if symbols.is_empty() {
        tracing::info!("No symbols returned from ChangeGuard index");
        return Ok(0);
    }

    #[cfg(feature = "graph")]
    let event_store = crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone());
    #[cfg(not(feature = "graph"))]
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    ingest_symbol_records(&event_store, project_id, project_root.as_deref(), symbols)
}

fn ingest_symbol_records(
    event_store: &dyn EventStore,
    project_id: ProjectId,
    project_root: Option<&std::path::Path>,
    symbols: Vec<SymbolRecord>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut ingested = 0usize;
    for symbol in symbols
        .into_iter()
        .filter(|symbol| symbol_in_project(&symbol.file_path, project_root))
        .take(500)
    {
        let namespace = Uuid::NAMESPACE_URL;
        let key = format!("{}:{}", project_id, symbol.qualified_name);
        let memory_uuid = Uuid::new_v5(&namespace, key.as_bytes());
        let memory_id = MemoryId::from_uuid(memory_uuid);

        if symbol_already_ingested(event_store, memory_uuid) {
            continue;
        }

        let ev = EventBuilder::new(
            AggregateType::Memory,
            memory_uuid,
            EventKind::MemoryPinned,
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: symbol_content(&symbol),
            session_id: None,
            project_id: Some(project_id),
            tx_id: None,
            rank: None,
            source_tag: Some("changeguard:symbol".to_string()),
            query_text: None,
        }));

        match ev {
            Ok(envelope) => {
                if let Err(e) = event_store.append_event(&envelope) {
                    tracing::warn!("Failed to store symbol memory: {}", e);
                } else {
                    ingested += 1;
                }
            }
            Err(e) => tracing::warn!("Failed to build symbol event: {}", e),
        }
    }

    Ok(ingested)
}

fn symbol_content(symbol: &SymbolRecord) -> String {
    match (&symbol.method, &symbol.path_pattern) {
        (Some(method), Some(path)) if !method.is_empty() && !path.is_empty() => format!(
            "route {} {} -> {} ({}:{})",
            method, path, symbol.qualified_name, symbol.file_path, symbol.line_start
        ),
        _ => format!(
            "{} {} ({}:{})",
            symbol.symbol_kind, symbol.qualified_name, symbol.file_path, symbol.line_start
        ),
    }
}

/// Call `changeguard index` to refresh the symbol index.
/// Non-fatal; logs a warning if unavailable.
fn refresh_changeguard_index() {
    #[allow(clippy::disallowed_methods)]
    match std::process::Command::new("changeguard")
        .arg("index")
        .output()
    {
        Ok(o) if o.status.success() => {
            eprintln!("[Nightly] ChangeGuard symbol index refreshed.");
        }
        Ok(o) => {
            eprintln!(
                "[Nightly] ChangeGuard index refresh non-fatal: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            );
        }
        Err(e) => {
            eprintln!("[Nightly] ChangeGuard not available for indexing: {}", e);
        }
    }
}

/// Query public project symbols from ChangeGuard's local index.
///
/// Current ChangeGuard bridge snapshots do not expose arbitrary graph-query
/// export, so T70 reads the indexed SQLite state directly. Route metadata is
/// joined from `api_routes` when that table exists.
#[allow(clippy::disallowed_methods)]
fn query_symbols_from_changeguard() -> Result<Vec<SymbolRecord>, Box<dyn std::error::Error>> {
    let db_path = std::env::current_dir()?.join(".changeguard/state/ledger.db");
    if !db_path.exists() {
        tracing::info!("ChangeGuard ledger DB not found at {}", db_path.display());
        return Ok(Vec::new());
    }

    let conn =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    if !sqlite_table_exists(&conn, "project_symbols")?
        || !sqlite_table_exists(&conn, "project_files")?
    {
        return Ok(Vec::new());
    }

    let has_routes = sqlite_table_exists(&conn, "api_routes")?;
    let sql = if has_routes {
        "SELECT pf.file_path,
                ps.qualified_name,
                ps.symbol_name,
                ps.symbol_kind,
                COALESCE(ps.line_start, 0),
                ar.method,
                ar.path_pattern
         FROM project_symbols ps
         JOIN project_files pf ON ps.file_id = pf.id
         LEFT JOIN api_routes ar
           ON ar.handler_symbol_id = ps.id
           OR (ar.handler_file_id = ps.file_id AND ar.handler_symbol_name = ps.symbol_name)
         WHERE COALESCE(ps.is_public, 0) != 0
            OR ps.entrypoint_kind IN ('ENTRYPOINT', 'HANDLER', 'PUBLIC_API')
         ORDER BY pf.file_path, ps.line_start, ps.qualified_name"
    } else {
        "SELECT pf.file_path,
                ps.qualified_name,
                ps.symbol_name,
                ps.symbol_kind,
                COALESCE(ps.line_start, 0),
                NULL,
                NULL
         FROM project_symbols ps
         JOIN project_files pf ON ps.file_id = pf.id
         WHERE COALESCE(ps.is_public, 0) != 0
            OR ps.entrypoint_kind IN ('ENTRYPOINT', 'HANDLER', 'PUBLIC_API')
         ORDER BY pf.file_path, ps.line_start, ps.qualified_name"
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(SymbolRecord {
            file_path: row.get(0)?,
            qualified_name: row.get(1)?,
            symbol_name: row.get(2)?,
            symbol_kind: row.get(3)?,
            line_start: row.get(4)?,
            method: non_empty(row.get::<_, Option<String>>(5)?.unwrap_or_default()),
            path_pattern: non_empty(row.get::<_, Option<String>>(6)?.unwrap_or_default()),
        })
    })?;

    let mut symbols = Vec::new();
    for row in rows {
        let symbol = row?;
        if !symbol.qualified_name.is_empty() {
            symbols.push(symbol);
        }
    }

    Ok(symbols)
}

fn sqlite_table_exists(conn: &rusqlite::Connection, table: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM sqlite_master
            WHERE type = 'table' AND name = ?1
        )",
        [table],
        |row| row.get::<_, bool>(0),
    )
}

fn symbol_already_ingested(event_store: &dyn EventStore, memory_uuid: Uuid) -> bool {
    event_store
        .read_events(memory_uuid)
        .map(|events| {
            events.iter().any(|event| match &event.payload {
                Payload::MemoryPinned(payload) => {
                    payload.source_tag.as_deref() == Some("changeguard:symbol")
                }
                _ => false,
            })
        })
        .unwrap_or(false)
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn symbol_in_project(file_path: &str, project_root: Option<&std::path::Path>) -> bool {
    let Some(project_root) = project_root else {
        return true;
    };
    let path = std::path::Path::new(file_path);
    if !path.is_absolute() {
        return true;
    }

    match (
        std::fs::canonicalize(path),
        std::fs::canonicalize(project_root),
    ) {
        (Ok(file), Ok(root)) => file.starts_with(root),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use ai_brains_retrieval::{recall, RecallOptions};
    use ai_brains_store::connection::VaultConnection;
    use ai_brains_store::event_store::SqliteEventStore;
    use tempfile::NamedTempFile;

    fn setup_store() -> Result<SqliteEventStore, Box<dyn std::error::Error>> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file
            .path()
            .to_str()
            .ok_or("invalid temp path")?
            .to_string();
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        let conn = VaultConnection::open(&db_path, &sql_key)?;
        conn.migrate()?;
        Ok(SqliteEventStore::new(conn))
    }

    #[test]
    fn route_symbol_content_includes_method_path_and_handler() {
        let symbol = SymbolRecord {
            file_path: "src/routes/user.rs".to_string(),
            qualified_name: "crate::routes::get_user".to_string(),
            symbol_name: "get_user".to_string(),
            symbol_kind: "Function".to_string(),
            line_start: 42,
            method: Some("GET".to_string()),
            path_pattern: Some("/users/:id".to_string()),
        };

        assert_eq!(
            symbol_content(&symbol),
            "route GET /users/:id -> crate::routes::get_user (src/routes/user.rs:42)"
        );
    }

    #[test]
    fn project_filter_rejects_absolute_paths_outside_root() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        let outside_file = outside.path().join("outside.rs");
        std::fs::write(&outside_file, "fn outside() {}")?;

        let outside_path = outside_file
            .to_str()
            .ok_or("invalid outside path")?
            .to_string();

        assert!(!symbol_in_project(&outside_path, Some(root.path())));
        assert!(symbol_in_project("src/lib.rs", Some(root.path())));
        Ok(())
    }

    #[test]
    fn symbol_ingestion_is_idempotent_and_recallable() -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbols = vec![SymbolRecord {
            file_path: "src/routes/user.rs".to_string(),
            qualified_name: "crate::routes::get_user".to_string(),
            symbol_name: "get_user".to_string(),
            symbol_kind: "Function".to_string(),
            line_start: 42,
            method: Some("GET".to_string()),
            path_pattern: Some("/users/:id".to_string()),
        }];

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, symbols.clone())?,
            1
        );
        assert_eq!(ingest_symbol_records(&store, project_id, None, symbols)?, 0);

        let hits = recall(
            store.connection(),
            None,
            "get_user",
            5,
            RecallOptions {
                project_id: Some(project_id),
                session_id: None,
                semantic: false,
                graph_boost: 0.0,
                graph_hop_depth: 0,
            },
        )?;

        assert!(hits.iter().any(|hit| {
            hit.content.contains("route GET /users/:id")
                && hit.content.contains("crate::routes::get_user")
        }));
        Ok(())
    }
}
