use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct BackupService {
    vault_path: PathBuf,
    custom_output: Option<PathBuf>,
}

impl BackupService {
    pub fn new(vault_path: PathBuf) -> Self {
        Self {
            vault_path,
            custom_output: None,
        }
    }

    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.custom_output = Some(dir);
        self
    }

    pub fn run_backup(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.vault_path.exists() {
            return Err("Vault file does not exist".into());
        }

        let parent = self.vault_path.parent().ok_or("Invalid vault path")?;
        let backup_dir = self
            .custom_output
            .clone()
            .unwrap_or_else(|| parent.join("backups"));
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%dT%H-%M-%S");
        let backup_path = backup_dir.join(format!("vault-{}.db.bak", timestamp));

        // Use SQLite backup API for consistent, safe backups
        let src = rusqlite::Connection::open(&self.vault_path)?;
        let mut dst = rusqlite::Connection::open(&backup_path)?;
        {
            let backup = rusqlite::backup::Backup::new(&src, &mut dst)?;
            backup.run_to_completion(10, std::time::Duration::from_millis(250), None)?;
        }

        // Verify integrity of the backup
        dst.execute_batch("PRAGMA integrity_check")?;

        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_run_backup() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");

        // Create a real SQLite database file so the backup API works
        let conn = rusqlite::Connection::open(&vault_path)?;
        conn.execute_batch(
            "CREATE TABLE test (id INTEGER PRIMARY KEY); INSERT INTO test VALUES (1);",
        )?;
        drop(conn);

        let service = BackupService::new(vault_path.clone());
        let backup_path = service.run_backup()?;

        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backups"));

        // Verify the backup has our table
        let backup_conn = rusqlite::Connection::open(&backup_path)?;
        let count: i32 =
            backup_conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
        assert_eq!(count, 1);

        Ok(())
    }
}
