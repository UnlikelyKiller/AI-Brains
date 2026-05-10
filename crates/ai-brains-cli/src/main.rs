mod commands;
mod context;
mod hotspot;

use crate::context::AppContext;
use ai_brains_core::ids::{ProjectId, SessionId};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ai-brains")]
#[command(about = "AI-Brains CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the vault database
    #[arg(long, env = "AI_BRAINS_VAULT_PATH")]
    vault_path: Option<PathBuf>,

    /// Hex-encoded key for the vault (or dummy)
    #[arg(long, env = "AI_BRAINS_KEY")]
    key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault
    Init,
    /// Ingest a conversation turn (reads JSON from stdin)
    Ingest,
    /// Recall memories based on a query
    Recall {
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        #[arg(long, env = "AI_BRAINS_SESSION_ID")]
        session_id: Option<SessionId>,
    },
    /// Generate preflight context for an LLM
    Preflight {
        #[arg(short, long, default_value_t = 1500)]
        max_words: usize,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
    },
    /// Run nightly intelligence sweep
    Nightly {
        /// Print the command to schedule this job on Windows
        #[arg(long)]
        schedule: bool,
        /// Start time for the scheduled task (e.g. "03:00")
        #[arg(long, default_value = "03:00")]
        start_time: String,
    },
    /// Create a timestamped backup of the vault
    Backup,
    /// Forget a specific memory (soft delete)
    Forget {
        /// Memory ID to forget
        memory_id: String,
    },
    /// Stop an active session
    StopSession {
        /// Session ID to stop
        session_id: String,
    },
    /// Initialize or refresh the project context (writes local .env)
    Context {
        /// Force a fresh project ID even if one is detected
        #[arg(long)]
        new_project: bool,
    },
    /// Pin a high-level decision or constraint directly to the vault
    Pin {
        /// The content to pin (e.g., "DECISION: Switched to SQLite")
        content: String,
        /// The role to associate with this pin (default: assistant)
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Privacy level (default: LocalOnly)
        #[arg(long, default_value = "LocalOnly")]
        privacy: String,
    },
    /// Manage repository safety signals
    Safety {
        #[command(subcommand)]
        command: SafetyCommands,
    },
    /// Import Antigravity conversation logs into the vault
    AntigravityImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
    },
}

#[derive(Subcommand, Clone)]
pub enum SafetyCommands {
    /// Synchronize ChangeGuard hotspots into the AI-Brains vault
    Sync {
        /// Limit the number of hotspots to ingest
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
}

fn main() {
    // Project .env always wins over inherited shell vars — prevents cross-project ID bleed
    dotenvy::dotenv_override().ok();

    // Fallback to global config in ~/.ai-brains/.env if AI_BRAINS_VAULT_PATH not set yet
    if std::env::var("AI_BRAINS_VAULT_PATH").is_err() {
        if let Some(mut home) = dirs::home_dir() {
            home.push(".ai-brains");
            home.push(".env");
            if home.exists() {
                dotenvy::from_path_override(home).ok();
            }
        }
    }

    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Context { new_project } => commands::context::run(*new_project),
        _ => {
            let ctx = AppContext::from_cli(cli.vault_path.clone(), cli.key.clone())?;
            match &cli.command {
                Commands::Init => commands::init::run(&ctx),
                Commands::Ingest => commands::ingest::run(&ctx),
                Commands::Recall {
                    query,
                    limit,
                    project_id,
                    session_id,
                } => commands::recall::run(&ctx, query.clone(), *limit, *project_id, *session_id),
                Commands::Preflight {
                    max_words,
                    project_id,
                } => commands::preflight::run(&ctx, *max_words, *project_id),
                Commands::Nightly {
                    schedule,
                    start_time,
                } => commands::nightly::run(&ctx, *schedule, start_time.clone()),
                Commands::Backup => commands::backup::run(&ctx),
                Commands::Forget { memory_id } => commands::forget::run(&ctx, memory_id.clone()),
                Commands::StopSession { session_id } => {
                    commands::stop_session::run(&ctx, session_id.clone())
                }
                Commands::Pin {
                    content,
                    role,
                    privacy,
                } => commands::pin::run(&ctx, content.clone(), role.clone(), privacy.clone()),
                Commands::Safety { command } => match command {
                    SafetyCommands::Sync { limit } => commands::safety::run(&ctx, *limit),
                },
                Commands::AntigravityImport { days } => {
                    commands::antigravity_import::run(&ctx, *days)
                }
                _ => unreachable!(),
            }
        }
    }
}
