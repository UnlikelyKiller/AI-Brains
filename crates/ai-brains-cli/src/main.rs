mod commands;
mod context;

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
        /// Output format: 'json' (default) or 'pretty'
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Generate preflight context for an LLM
    Preflight {
        #[arg(short, long, default_value_t = 1500)]
        max_words: usize,
        #[arg(long, env = "AI_BRAINS_PROJECT_ID")]
        project_id: Option<ProjectId>,
        /// Output human-readable text instead of JSON
        #[arg(long)]
        pretty: bool,
    },
    /// Run nightly intelligence sweep
    Nightly {
        /// Schedule this as a Windows scheduled task
        #[arg(long)]
        schedule: bool,
        /// Remove the Windows scheduled task
        #[arg(long)]
        unschedule: bool,
        /// Start time for the scheduled task (e.g. "03:00")
        #[arg(long, default_value = "03:00")]
        start_time: String,
    },
    /// Create a timestamped backup of the vault
    Backup {
        #[command(subcommand)]
        command: Option<BackupCommands>,
    },
    /// Forget a specific memory (soft delete)
    Forget {
        /// Memory ID to forget
        #[arg(long)]
        memory_id: Option<String>,
        /// Search for memories by content match
        #[arg(long = "match")]
        match_query: Option<String>,
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,
        /// List all forgotten memories
        #[arg(long)]
        list_forgotten: bool,
        /// Restore a forgotten memory
        #[arg(long)]
        restore: Option<String>,
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
        /// Force a new session ID, replacing the existing one
        #[arg(long)]
        new_session: bool,
        /// Show current context without modifying anything
        #[arg(long)]
        show: bool,
        /// Optional ChangeGuard transaction ID to link this context to
        #[arg(long, env = "CHANGEGUARD_TX_ID")]
        tx_id: Option<String>,
    },
    /// Pin a high-level decision or constraint directly to the vault
    Pin {
        /// The content to pin (e.g., "DECISION: Switched to SQLite")
        content: Option<String>,
        /// The role to associate with this pin (default: assistant)
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Privacy level (default: LocalOnly)
        #[arg(long, default_value = "LocalOnly")]
        privacy: String,
        /// Read content from stdin instead of positional arg
        #[arg(long)]
        stdin: bool,
        /// Tags to categorize this memory (repeatable)
        #[arg(long = "tag", short = 't')]
        tags: Vec<String>,
        /// Optional ChangeGuard transaction ID to link this pin to
        #[arg(long, env = "CHANGEGUARD_TX_ID")]
        tx_id: Option<String>,
    },
    /// Manage repository safety signals
    Safety {
        #[command(subcommand)]
        command: SafetyCommands,
    },
    /// Sync structured records from external tools (ChangeGuard)
    Sync {
        #[command(subcommand)]
        command: SyncCommands,
    },
    /// Import Antigravity conversation logs into the vault
    AntigravityImport {
        /// Only import sessions modified within the last N days
        #[arg(short, long, default_value_t = 30)]
        days: usize,
    },
}

#[derive(Subcommand, Clone)]
pub enum BackupCommands {
    /// Create a timestamped backup (default)
    Create {
        /// Custom output directory for the backup
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Restore vault from a backup file
    Restore {
        /// Path to the backup file
        path: PathBuf,
    },
}

#[derive(Subcommand, Clone)]
pub enum SyncCommands {
    /// Pull records from an NDJSON file
    Pull {
        /// Path to the NDJSON file
        #[arg(long)]
        from_file: PathBuf,
    },
}

#[derive(Subcommand, Clone)]
pub enum SafetyCommands {
    /// Synchronize ChangeGuard hotspots into the AI-Brains vault
    Sync {
        /// Limit the number of hotspots to ingest
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        /// Preview what would be synced without pinning
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    // Project .env always wins over inherited shell vars.
    // If no local .env exists, we clear project-specific env vars to prevent
    // stale inheritance from other projects in the same shell session.
    if !std::path::Path::new(".env").exists() {
        std::env::remove_var("AI_BRAINS_PROJECT_ID");
        std::env::remove_var("AI_BRAINS_SESSION_ID");
    } else {
        dotenvy::dotenv_override().ok();
    }

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
        Commands::Context {
            new_project,
            new_session,
            show,
            tx_id,
        } => commands::context::run(*new_project, *new_session, *show, tx_id.clone()),
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
                    format,
                } => commands::recall::run(
                    &ctx,
                    query.clone(),
                    *limit,
                    *project_id,
                    *session_id,
                    format.clone(),
                ),
                Commands::Preflight {
                    max_words,
                    project_id,
                    pretty,
                } => commands::preflight::run(&ctx, *max_words, *project_id, *pretty),
                Commands::Nightly {
                    schedule,
                    unschedule,
                    start_time,
                } => commands::nightly::run(&ctx, *schedule, *unschedule, start_time.clone()),
                Commands::Backup { command } => match command {
                    Some(BackupCommands::Restore { path }) => {
                        commands::backup::run_restore(&ctx, path.clone())
                    }
                    Some(BackupCommands::Create { output_dir }) => {
                        commands::backup::run_create(&ctx, output_dir.clone())
                    }
                    None => commands::backup::run_create(&ctx, None),
                },
                Commands::Forget {
                    memory_id,
                    match_query,
                    force,
                    list_forgotten,
                    restore,
                } => commands::forget::run(
                    &ctx,
                    memory_id.clone(),
                    match_query.clone(),
                    *force,
                    *list_forgotten,
                    restore.clone(),
                ),
                Commands::StopSession { session_id } => {
                    commands::stop_session::run(&ctx, session_id.clone())
                }
                Commands::Pin {
                    content,
                    role,
                    privacy,
                    stdin,
                    tags,
                    tx_id,
                } => {
                    if *stdin {
                        commands::pin::run_stdin(
                            &ctx,
                            role.clone(),
                            privacy.clone(),
                            tags.clone(),
                            tx_id.clone(),
                        )
                    } else if let Some(c) = content {
                        commands::pin::run(
                            &ctx,
                            c.clone(),
                            role.clone(),
                            privacy.clone(),
                            tags.clone(),
                            tx_id.clone(),
                        )
                    } else {
                        Err("Either provide content as a positional argument or use --stdin to read from stdin.".into())
                    }
                }
                Commands::Safety { command } => match command {
                    SafetyCommands::Sync { limit, dry_run } => {
                        commands::safety::run(&ctx, *limit, *dry_run)
                    }
                },
                Commands::Sync { command } => match command {
                    SyncCommands::Pull { from_file } => {
                        commands::sync::run_pull(&ctx, from_file.clone())
                    }
                },
                Commands::AntigravityImport { days } => {
                    commands::antigravity_import::run(&ctx, *days)
                }
                _ => unreachable!(),
            }
        }
    }
}
