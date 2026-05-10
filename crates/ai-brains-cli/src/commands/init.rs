use crate::context::AppContext;

pub fn run(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Vault initialized successfully at {}",
        ctx.vault_path.display()
    );
    Ok(())
}
