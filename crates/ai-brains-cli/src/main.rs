mod commands;
mod context;
mod daemon_client;
mod live_graph;

use crate::context::AppContext;
use ai_brains_core::ids::{ProjectId, SessionId};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ai-brains")]
#[command(version)]
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
        /// Use semantic (embedding) search alongside FTS5
        #[arg(long)]
        semantic: bool,
        /// Score boost added to graph-neighbor hits (default 0.1)
        #[arg(long, default_value_t = 0.1)]
        graph_boost: f64,
        /// Hop depth for graph expansion (reserved; currently only depth=1)
        #[arg(long, default_value_t = 1)]
        graph_hop_depth: usize,
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
        /// Output format: 'json' or 'human'
        #[arg(long)]
        format: Option<String>,
        /// Comma-separated target file/directory paths for contextual risk analysis
        #[arg(long, env = "AI_BRAINS_SCOPE", value_delimiter = ',')]
        scope: Vec<String>,
        /// Output a concise statistical summary instead of full text
        #[arg(short, long)]
        summary: bool,
        /// Aggregate context across ALL projects (ignores project_id filter)
        #[arg(long)]
        global: bool,
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
        /// Show read-only status of the last nightly run and pending work
        #[arg(long, conflicts_with = "schedule", conflicts_with = "unschedule")]
        status: bool,
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
    /// Process an Antigravity CLI (agy) hook payload
    AgyHook {
        /// The JSON payload from agy
        #[arg(long)]
        payload: String,
    },
    /// Manage the AI-Brains daemon process
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Manage projects and resolve aliases
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    #[cfg(feature = "graph")]
    /// Graph operations (rebuild, query)
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
}

#[derive(Subcommand, Clone)]
pub enum GraphCommands {
    /// Rebuild graph from all events
    Rebuild,
    /// Show 1-hop graph neighbors of a memory
    Neighbors {
        memory_id: String,
    },
    /// Show recursive SYNTHESIZED_FROM hierarchy of a memory
    Hierarchy {
        memory_id: String,
    },
    /// Show all memories in a session via graph edges
    Session {
        session_id: String,
    },
    /// Show current graph health: node/edge counts
    Update,
}

#[derive(Subcommand, Clone)]
pub enum ProjectCommands {
    /// List all projects in the vault
    List,
    /// Resolve an alias to a project ID
    Resolve {
        alias: String,
    },
    /// Auto-detect project from current git repository
    Detect {
        /// Output as shell export statement
        #[arg(long)]
        export: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum DaemonCommands {
    /// Stop the running daemon gracefully
    Stop {
        /// Forcefully terminate the process if it doesn't respond to shutdown signal
        #[arg(long, short)]
        force: bool,
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
        from_file: Option<PathBuf>,
        /// Export hotspot data from ChangeGuard
        #[arg(long)]
        hotspots: bool,
        /// Export ledger delta data from ChangeGuard
        #[arg(long)]
        ledger: bool,
        /// Suppress ChangeGuard error messages
        #[arg(long, short)]
        quiet: bool,
    },
    /// Push current context to ChangeGuard
    Push {
        /// Include impact context
        #[arg(long)]
        with_impact: bool,
        /// Include verification context
        #[arg(long)]
        with_verify: bool,
        /// Suppress ChangeGuard error messages
        #[arg(long, short)]
        quiet: bool,
    },
    /// Unified query across AI-Brains and ChangeGuard
    Query {
        /// The query string
        query: String,
        /// Output format (pretty, text, ndjson)
        #[arg(long)]
        format: Option<String>,
        /// Suppress daemon-down error messages
        #[arg(long, short)]
        quiet: bool,
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

    // Set up a basic signal handler for graceful interruption
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to initialize Tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    runtime.block_on(async {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nInterrupted by user. Exiting...");
                std::process::exit(130);
            }
            res = async {
                let cli = Cli::parse();
                run(cli).await
            } => {
                if let Err(err) = res {
                    use ai_brains_contracts::response::{ApiError, ApiResult};
                    let api_error = ApiError::new("COMMAND_FAILED", err.to_string());
                    let result = ApiResult::<serde_json::Value>::error(api_error);
                    if let Ok(json) = serde_json::to_string(&result) {
                        eprintln!("{}", json);
                    } else {
                        eprintln!("Error: {err}");
                    }
                    std::process::exit(1);
                }
            }
        }
    });
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
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
            semantic,
            graph_boost,
            graph_hop_depth,
        } => commands::recall::run(
            &ctx,
            query.clone(),
            *limit,
            *project_id,
            *session_id,
            format.clone(),
            *semantic,
            *graph_boost,
            *graph_hop_depth,
        ),
        Commands::Preflight {
            max_words,
            project_id,
            pretty,
            format,
            scope,
            summary,
            global,
        } => commands::preflight::run(
            &ctx,
            *max_words,
            *project_id,
            *pretty,
            format.clone(),
            scope.clone(),
            *summary,
            *global,
        ),
        Commands::Nightly {
            schedule,
            unschedule,
            start_time,
            status,
        } => {
            commands::nightly::run(&ctx, *schedule, *unschedule, start_time.clone(), *status).await
        }
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
        Commands::Context {
            new_project,
            new_session,
            show,
            tx_id,
        } => commands::context::run(&ctx, *new_project, *new_session, *show, tx_id.clone()),
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
            SyncCommands::Pull {
                from_file,
                hotspots,
                ledger,
                quiet,
            } => commands::sync::run_pull(&ctx, from_file.clone(), *hotspots, *ledger, *quiet),
            SyncCommands::Push {
                with_impact,
                with_verify,
                quiet,
            } => commands::sync::run_push(&ctx, *with_impact, *with_verify, *quiet),
            SyncCommands::Query {
                query,
                format,
                quiet,
            } => commands::sync::run_query(&ctx, query.clone(), format.clone(), *quiet).await,
        },
        Commands::AntigravityImport { days } => commands::antigravity_import::run(&ctx, *days),
        Commands::AgyHook { payload } => commands::agy_hook::run(&ctx, payload),
        Commands::Daemon { command } => match command {
            DaemonCommands::Stop { force } => commands::daemon::run_stop(&ctx, *force).await,
        },
        Commands::Project { command } => match command {
            ProjectCommands::List => commands::project::list(&ctx),
            ProjectCommands::Resolve { alias } => commands::project::resolve(&ctx, alias),
            ProjectCommands::Detect { export } => commands::project::detect(&ctx, *export),
        },
        #[cfg(feature = "graph")]
        Commands::Graph { command } => match command {
            GraphCommands::Rebuild => commands::graph::rebuild(&ctx),
            GraphCommands::Neighbors { memory_id } => {
                commands::graph::neighbors(&ctx, memory_id)
            }
            GraphCommands::Hierarchy { memory_id } => {
                commands::graph::hierarchy(&ctx, memory_id)
            }
            GraphCommands::Session { session_id } => {
                commands::graph::session(&ctx, session_id)
            }
            GraphCommands::Update => commands::graph::update(&ctx),
        },
    }
}
