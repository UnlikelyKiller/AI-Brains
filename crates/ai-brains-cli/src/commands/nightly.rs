use crate::context::AppContext;
use std::sync::Arc;

pub fn run(
    ctx: &AppContext,
    schedule: bool,
    start_time: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if schedule {
        let exe_path = std::env::current_exe()?;
        let cmd = ai_brains_scheduler::TaskScheduler::render_create_command(
            exe_path.to_str().ok_or("Invalid executable path")?,
            "AI-Brains-Nightly",
            &start_time,
        );
        println!("Run the following command in an elevated PowerShell session to schedule the nightly job:");
        println!("\n{}\n", cmd);
        return Ok(());
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

    println!("Starting nightly intelligence sweep...");
    let tokio_runtime = tokio::runtime::Runtime::new()?;
    let count = tokio_runtime.block_on(service.run_nightly())?;
    println!("Nightly sweep completed. Processed {} sessions.", count);

    Ok(())
}
