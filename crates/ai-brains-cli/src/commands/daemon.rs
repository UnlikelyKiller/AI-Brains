use crate::context::AppContext;
use crate::daemon_client::DaemonClient;

pub async fn run_stop(_ctx: &AppContext, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();

    if force {
        eprintln!("Forcefully stopping AI-Brains daemon...");
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/IM", "ai-brainsd.exe"])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("pkill")
                .arg("ai-brainsd")
                .output();
        }
        println!("Daemon stopped (forced).");
        return Ok(());
    }

    eprintln!("Sending shutdown signal to AI-Brains daemon...");
    match client.shutdown().await {
        Ok(_) => {
            println!("Shutdown signal sent successfully.");
            // Give it a moment to exit
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        Err(e) => {
            eprintln!(
                "Failed to send shutdown signal: {}. The daemon might not be running.",
                e
            );
            eprintln!("Use --force to kill the process if it's unresponsive.");
        }
    }

    Ok(())
}
