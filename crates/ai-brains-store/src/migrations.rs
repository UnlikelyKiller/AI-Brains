use crate::errors::{Result, StoreError};
use rusqlite::Connection;

pub const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_event_log",
        include_str!("../migrations/0001_event_log.sql"),
    ),
    (
        "0002_identity_projection",
        include_str!("../migrations/0002_identity_projection.sql"),
    ),
    (
        "0003_project_projection",
        include_str!("../migrations/0003_project_projection.sql"),
    ),
    (
        "0004_session_projection",
        include_str!("../migrations/0004_session_projection.sql"),
    ),
    (
        "0005_turn_projection",
        include_str!("../migrations/0005_turn_projection.sql"),
    ),
    (
        "0006_memory_projection",
        include_str!("../migrations/0006_memory_projection.sql"),
    ),
    (
        "0007_fts_setup",
        include_str!("../migrations/0007_fts_setup.sql"),
    ),
    (
        "0008_fts_triggers",
        include_str!("../migrations/0008_fts_triggers.sql"),
    ),
    (
        "0009_session_summarization",
        include_str!("../migrations/0009_session_summarization.sql"),
    ),
    (
        "0010_conflict_recipe_projection",
        include_str!("../migrations/0010_conflict_recipe_projection.sql"),
    ),
    (
        "0011_memory_hierarchy",
        include_str!("../migrations/0011_memory_hierarchy.sql"),
    ),
    (
        "0012_retention_support",
        include_str!("../migrations/0012_retention_support.sql"),
    ),
    (
        "0013_relational_graph",
        include_str!("../migrations/0013_relational_graph.sql"),
    ),
    (
        "0014_memory_session_context",
        include_str!("../migrations/0014_memory_session_context.sql"),
    ),
    (
        "0015_memory_project_id",
        include_str!("../migrations/0015_memory_project_id.sql"),
    ),
];

pub fn apply_migrations(conn: &mut Connection) -> Result<()> {
    // Create schema_migrations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    for (name, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = ?)",
            [name],
            |row| row.get(0),
        )?;

        if !already_applied {
            let tx = conn.transaction()?;

            tx.execute_batch(sql).map_err(|e| {
                StoreError::MigrationFailed(format!("Failed to apply migration {}: {}", name, e))
            })?;

            tx.execute("INSERT INTO schema_migrations (name) VALUES (?)", [name])?;

            tx.commit()?;
        }
    }

    Ok(())
}
