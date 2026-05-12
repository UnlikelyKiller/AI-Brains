use crate::context::AppContext;
use std::path::PathBuf;

pub fn run_create(
    ctx: &AppContext,
    output_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut service = ai_brains_brain::BackupService::new(ctx.vault_path.clone());
    if let Some(dir) = output_dir {
        service = service.with_output_dir(dir);
    }
    eprintln!("Creating vault backup...");
    let backup_path = service.run_backup()?;
    println!("Backup created and verified: {}", backup_path.display());
    Ok(())
}

pub fn run_restore(
    ctx: &AppContext,
    backup_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if !backup_path.exists() {
        return Err(format!("Backup file not found: {}", backup_path.display()).into());
    }

    // Verify integrity of the backup before restoring
    let bak_conn = rusqlite::Connection::open(&backup_path)?;
    bak_conn.execute_batch("PRAGMA integrity_check")?;

    eprintln!(
        "WARNING: This will overwrite the current vault at {}",
        ctx.vault_path.display()
    );
    eprint!("Type 'yes' to continue: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "yes" {
        return Err("Restore cancelled.".into());
    }

    // Restore via SQLite backup API (overwrites current vault)
    let mut vault_conn = rusqlite::Connection::open(&ctx.vault_path)?;
    let backup = rusqlite::backup::Backup::new(&bak_conn, &mut vault_conn)?;
    backup.run_to_completion(10, std::time::Duration::from_millis(250), None)?;

    println!("Vault restored from: {}", backup_path.display());
    Ok(())
}
