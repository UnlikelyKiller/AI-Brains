use crate::context::AppContext;

pub fn run(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let service = ai_brains_brain::BackupService::new(ctx.vault_path.clone());
    println!("Creating vault backup...");
    let backup_path = service.run_backup()?;
    println!("Backup created successfully: {}", backup_path.display());
    Ok(())
}
