use crate::context::AppContext;

pub fn run(ctx: &AppContext, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for ChangeGuard Hotspots...");

    let output = std::process::Command::new("changeguard")
        .args(["hotspots", "--limit", &limit.to_string()])
        .output()?;

    if !output.status.success() {
        return Err(
            "ChangeGuard scan failed. Ensure ChangeGuard is installed and initialized.".into(),
        );
    }

    let raw_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw_output.is_empty() {
        println!("No hotspots identified. Safety layer is healthy.");
        return Ok(());
    }

    let hotspots = crate::hotspot::sanitize_and_condense(&raw_output);
    if hotspots.is_empty() {
        println!("No hotspots identified. Safety layer is healthy.");
        return Ok(());
    }

    println!("Ingesting hotspots into AI-Brains vault...");

    let content = format!(
        "HOTSPOT: Brittle files identified by ChangeGuard:\n\n{}",
        hotspots
    );

    super::pin::run(
        ctx,
        content,
        "assistant".to_string(),
        "LocalOnly".to_string(),
    )?;

    println!("Safety synchronization complete.");
    Ok(())
}
