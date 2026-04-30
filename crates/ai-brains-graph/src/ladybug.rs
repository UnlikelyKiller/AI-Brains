use crate::errors::{GraphError, Result};
use lbug::{Connection, Database, SystemConfig};
use std::path::Path;

pub struct LadybugVault {
    db: Database,
}

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
