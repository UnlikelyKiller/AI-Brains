use crate::context::AppContext;
use ai_brains_core::ids::ProjectId;
use std::str::FromStr;
use std::sync::Arc;

pub fn run(
    ctx: &AppContext,
    schedule: bool,
    unschedule: bool,
    start_time: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let task_name = "AI-Brains-Nightly";

    if unschedule {
        let output = std::process::Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .output()
            .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

        if output.status.success() {
            println!("Nightly task '{}' removed.", task_name);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Failed to remove task: {}", stderr);
        }
        return Ok(());
    }

    if schedule {
        let exe_path = std::env::current_exe()?;
        let exe_str = exe_path.to_str().ok_or("Invalid executable path")?;

        let output = std::process::Command::new("schtasks")
            .args([
                "/create",
                "/tn",
                task_name,
                "/tr",
                &format!("'{}' nightly", exe_str),
                "/sc",
                "daily",
                "/st",
                &start_time,
                "/f",
            ])
            .output()
            .map_err(|e| {
                format!(
                    "Failed to execute schtasks: {}. Run in an elevated PowerShell session.",
                    e
                )
            })?;

        if output.status.success() {
            println!(
                "Nightly task '{}' scheduled daily at {}.",
                task_name, start_time
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let cmd = ai_brains_scheduler::TaskScheduler::render_create_command(
                exe_str,
                task_name,
                &start_time,
            );
            eprintln!(
                "Failed to schedule task. Run this in an elevated PowerShell session:\n{}\nError: {}{}",
                cmd, stdout, stderr
            );
        }
        return Ok(());
    }

    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default();

    if project_id == ProjectId::default() {
        eprintln!(
            "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first. Using default project."
        );
    }

    let event_store = Arc::new(ai_brains_store::SqliteEventStore::new((*ctx.conn).clone()));
    let query_store = ctx.conn.clone() as Arc<dyn ai_brains_store::QueryStore>;

    let model_url = std::env::var("AI_BRAINS_MODEL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
    let completion_model =
        std::env::var("AI_BRAINS_COMPLETION_MODEL").unwrap_or_else(|_| "qwen3.5-9b".to_string());
    let embedding_model =
        std::env::var("AI_BRAINS_EMBEDDING_MODEL").unwrap_or_else(|_| "bge-m3".to_string());

    let completion_provider = Arc::new(ai_brains_models::llama_cpp::LlamaCppProvider::new(
        model_url.clone(),
        completion_model,
    ));
    let embedding_provider = Arc::new(ai_brains_models::llama_cpp::LlamaCppProvider::new(
        model_url,
        embedding_model,
    ));

    // Import Antigravity sessions before summarization so they get summarized too
    if let Err(e) = crate::commands::antigravity_import::run(ctx, 30) {
        tracing::error!("Antigravity import failed: {}", e);
    }

    let service = ai_brains_brain::NightlyService::new(
        query_store,
        event_store,
        completion_provider,
        embedding_provider,
    );

    eprintln!("Starting nightly intelligence sweep...");
    eprintln!("Summarizing sessions...");
    let tokio_runtime = tokio::runtime::Runtime::new()?;

    let count = tokio_runtime.block_on(service.run_nightly(project_id))?;
    eprintln!("Running memory synthesis...");
    eprintln!("Nightly sweep completed. {} sessions summarized.", count);

    Ok(())
}
