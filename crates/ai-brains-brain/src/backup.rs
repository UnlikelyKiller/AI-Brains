use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct BackupService {
    vault_path: PathBuf,
}

impl BackupService {
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    pub fn run_backup(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.vault_path.exists() {
            return Err("Vault file does not exist".into());
        }

        let parent = self.vault_path.parent().ok_or("Invalid vault path")?;
        let backup_dir = parent.join("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        let now = Utc::now();
        let timestamp = now.to_rfc3339().replace(":", "-");
        let backup_name = format!("vault-{}.db.bak", timestamp);
        let backup_path = backup_dir.join(backup_name);

        fs::copy(&self.vault_path, &backup_path)?;

        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_backup() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        fs::write(&vault_path, "vault content")?;

        let service = BackupService::new(vault_path.clone());
        let backup_path = service.run_backup()?;

        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backups"));
        assert_eq!(fs::read_to_string(backup_path)?, "vault content");

        Ok(())
    }
}
