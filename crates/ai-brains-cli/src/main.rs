use ai_brains_capture::{parse_ingest_request, CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestResponse;
use ai_brains_contracts::preflight::PreflightContextResponse;
use ai_brains_contracts::recall::{RecallResponse, RecallResult};
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, EventKind, Payload, ProjectRegisteredPayload};
use ai_brains_retrieval::{build_preflight, recall};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::EventStore;
use clap::{Parser, Subcommand};
use rusqlite::params;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

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
}

#[derive(Subcommand)]
enum SafetyCommands {
    /// Synchronize ChangeGuard hotspots into the AI-Brains vault
    Sync {
        /// Limit the number of hotspots to ingest
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
}

fn main() {
    dotenvy::dotenv().ok();

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
        Commands::Init => run_init(&cli),
        Commands::Ingest => run_ingest(&cli),
        Commands::Recall {
            query,
            limit,
            project_id,
            session_id,
        } => run_recall(&cli, query.clone(), *limit, *project_id, *session_id),
        Commands::Preflight {
            max_words,
            project_id,
        } => run_preflight(&cli, *max_words, *project_id),
        Commands::Nightly {
            schedule,
            start_time,
        } => run_nightly(&cli, *schedule, start_time.clone()),
        Commands::Backup => run_backup(&cli),
        Commands::Forget { memory_id } => run_forget(&cli, memory_id.clone()),
        Commands::StopSession { session_id } => run_stop_session(&cli, session_id.clone()),
        Commands::Context { new_project } => run_context(&cli, *new_project),
        Commands::Pin {
            content,
            role,
            privacy,
        } => run_pin(&cli, content.clone(), role.clone(), privacy.clone()),
        Commands::Safety { command } => match command {
            SafetyCommands::Sync { limit } => run_safety_sync(&cli, *limit),
        },
    }
}

fn run_context(_cli: &Cli, new_project: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project");

    let project_id = if new_project {
        ProjectId::new()
    } else {
        // Deterministic project ID based on the canonical directory path
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&current_dir.to_string_lossy().to_lowercase(), &mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        // Use the hash to seed a UUID (this is a simplified approach)
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&hash.to_be_bytes());
        ProjectId::from_uuid(uuid::Uuid::from_bytes(bytes))
    };

    let session_id = SessionId::new();
    let harness_id = HarnessId::new();

    let env_content = format!(
        "AI_BRAINS_PROJECT_ID={}\nAI_BRAINS_SESSION_ID={}\nAI_BRAINS_HARNESS_ID={}\n",
        project_id, session_id, harness_id
    );

    let env_path = current_dir.join(".env");
    let mut final_content = if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path)?;
        // Simple filter to remove existing AI_BRAINS keys to avoid duplicates
        existing
            .lines()
            .filter(|l| {
                !l.starts_with("AI_BRAINS_PROJECT_ID")
                    && !l.starts_with("AI_BRAINS_SESSION_ID")
                    && !l.starts_with("AI_BRAINS_HARNESS_ID")
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    if !final_content.is_empty() && !final_content.ends_with('\n') {
        final_content.push('\n');
    }
    final_content.push_str(&env_content);

    std::fs::write(&env_path, final_content)?;

    println!("Context initialized for project: {}", project_name);
    println!("Project ID: {}", project_id);
    println!("Session ID: {}", session_id);
    println!("Local .env updated successfully.");

    Ok(())
}

fn open_vault(cli: &Cli) -> Result<VaultConnection, Box<dyn std::error::Error>> {
    let path = cli
        .vault_path
        .clone()
        .ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;

    // In degraded mode, we use a fixed dummy key if none provided
    // This allows rusqlite-bundled to work even if SQLCipher isn't active
    let key_str = cli.key.clone().unwrap_or_else(|| {
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
    });

    let key = SqlCipherKey::from_raw(key_str);
    let conn = VaultConnection::open(path, &key)?;
    conn.migrate()?;
    Ok(conn)
}

fn run_init(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let path = cli
        .vault_path
        .clone()
        .ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let _conn = open_vault(cli)?;
    println!("Vault initialized successfully at {}", path.display());
    Ok(())
}

fn run_ingest(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request = parse_ingest_request(&input)?;

    let conn = open_vault(cli)?;
    let event_store = ai_brains_store::SqliteEventStore::new(conn.clone());

    let mut sink = struct_sink::StoreSink {
        store: event_store,
        last_error: None,
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    // Auto-create project if it doesn't exist
    let project_exists = {
        let conn_lock = conn.lock()?;
        let mut stmt =
            conn_lock.prepare("SELECT 1 FROM project_projection WHERE project_id = ?")?;
        stmt.exists(params![request.project_id.to_string()])?
    };

    if !project_exists {
        let event = EventBuilder::new(
            AggregateType::Project,
            request.project_id.as_uuid(),
            EventKind::ProjectRegistered,
            Actor::User(ai_brains_core::ids::UserId::new()),
            request.privacy,
        )
        .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
            project_id: request.project_id,
            name: format!("Project {}", request.project_id),
        }))?;

        sink.store.append_event(&event)?;
    }

    // Auto-start session if it doesn't exist
    let session_exists = {
        let conn_lock = conn.lock()?;
        let mut stmt =
            conn_lock.prepare("SELECT 1 FROM session_projection WHERE session_id = ?")?;
        stmt.exists(params![request.session_id.to_string()])?
    };

    if !session_exists {
        service.start_session(
            ai_brains_capture::SessionStartCommand {
                session_id: request.session_id,
                project_id: request.project_id,
                harness_id: request.harness_id,
                privacy: request.privacy,
            },
            capture_context.clone(),
            &mut sink,
        )?;

        if let Some(err) = sink.last_error.take() {
            return Err(format!("Failed to auto-start session: {}", err).into());
        }
    }

    let outcome = service.ingest_request(request, capture_context, &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist turn: {}", err).into());
    }

    let response = IngestResponse {
        event_id: outcome.events[0].event_id.to_string(),
        processed: true,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn run_recall(
    cli: &Cli,
    query: String,
    limit: usize,
    project_id: Option<ProjectId>,
    session_id: Option<SessionId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_vault(cli)?;

    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new(conn.clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let hits = recall(
        &conn,
        graph_search.as_ref(),
        &query,
        limit,
        project_id,
        session_id,
    )?;

    let response = RecallResponse {
        results: hits
            .into_iter()
            .map(|h| RecallResult {
                memory_id: h.memory_id,
                content: h.content,
                source: h.source,
            })
            .collect(),
    };

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn run_preflight(
    cli: &Cli,
    max_words: usize,
    project_id: Option<ProjectId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_vault(cli)?;

    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new(conn.clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let context = build_preflight(&conn, graph_search.as_ref(), max_words, project_id)?;

    let response = PreflightContextResponse {
        text: context.text,
        word_count: context.word_count,
    };

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn run_nightly(
    cli: &Cli,
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

    let conn = open_vault(cli)?;
    let event_store = std::sync::Arc::new(ai_brains_store::SqliteEventStore::new(conn.clone()));
    let query_store = std::sync::Arc::new(conn);

    let model_url = std::env::var("AI_BRAINS_MODEL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
    let completion_model =
        std::env::var("AI_BRAINS_COMPLETION_MODEL").unwrap_or_else(|_| "qwen3.5-9b".to_string());
    let embedding_model =
        std::env::var("AI_BRAINS_EMBEDDING_MODEL").unwrap_or_else(|_| "bge-m3".to_string());

    let completion_provider = std::sync::Arc::new(
        ai_brains_models::llama_cpp::LlamaCppProvider::new(model_url.clone(), completion_model),
    );
    let embedding_provider = std::sync::Arc::new(
        ai_brains_models::llama_cpp::LlamaCppProvider::new(model_url, embedding_model),
    );

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

fn run_backup(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let path = cli
        .vault_path
        .clone()
        .ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;

    let service = ai_brains_brain::BackupService::new(path);
    println!("Creating vault backup...");
    let backup_path = service.run_backup()?;
    println!("Backup created successfully: {}", backup_path.display());
    Ok(())
}

fn run_forget(cli: &Cli, memory_id_str: String) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::ids::MemoryId;
    use ai_brains_core::privacy::Privacy;
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::{Actor, AggregateType, EventKind, MemoryForgottenPayload, Payload};

    let memory_id = MemoryId::from_str(&memory_id_str)?;
    let conn = open_vault(cli)?;
    let event_store = ai_brains_store::SqliteEventStore::new(conn.clone());

    let event = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryForgotten,
        Actor::User(ai_brains_core::ids::UserId::new()),
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryForgotten(MemoryForgottenPayload {
        memory_id,
    }))?;

    event_store.append_event(&event)?;

    // Projections will be updated by the next query or background job
    // But for immediate feedback, we can trigger a manual projection update if needed
    // Or just say it's done.

    println!("Memory {} marked as forgotten.", memory_id);
    Ok(())
}

fn run_stop_session(cli: &Cli, session_id_str: String) -> Result<(), Box<dyn std::error::Error>> {
    let session_id = SessionId::from_str(&session_id_str)?;
    let conn = open_vault(cli)?;
    let event_store = ai_brains_store::SqliteEventStore::new(conn.clone());

    let mut sink = struct_sink::StoreSink {
        store: event_store,
        last_error: None,
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    service.stop_session(
        ai_brains_capture::SessionStopCommand {
            session_id,
            harness_id: HarnessId::new(), // In a real scenario, this should come from context
            privacy: ai_brains_core::privacy::Privacy::LocalOnly,
            status: ai_brains_capture::SessionStopStatus::Completed,
            reason: None,
        },
        capture_context,
        &mut sink,
    )?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to stop session: {}", err).into());
    }

    println!("Session {} marked as completed.", session_id);
    Ok(())
}

fn run_pin(
    cli: &Cli,
    content: String,
    role: String,
    privacy_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_contracts::ingest::IngestRequest;
    use ai_brains_core::privacy::Privacy;

    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .map_err(|_| "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first.")?
        .parse::<ProjectId>()?;

    let session_id = std::env::var("AI_BRAINS_SESSION_ID")
        .map_err(|_| "AI_BRAINS_SESSION_ID not set. Run 'ai-brains context' first.")?
        .parse::<SessionId>()?;

    let harness_id = std::env::var("AI_BRAINS_HARNESS_ID")
        .ok()
        .and_then(|s| s.parse::<HarnessId>().ok())
        .unwrap_or_default();

    let privacy = match privacy_str.to_lowercase().as_str() {
        "cloudok" => Privacy::CloudOk,
        "localonly" => Privacy::LocalOnly,
        "neverinject" => Privacy::NeverInject,
        "sealed" => Privacy::Sealed,
        _ => Privacy::LocalOnly,
    };

    let request = IngestRequest {
        session_id,
        project_id,
        harness_id,
        turn_id: ai_brains_core::ids::TurnId::new(),
        role,
        content,
        thinking: None,
        privacy,
    };

    let conn = open_vault(cli)?;
    let event_store = ai_brains_store::SqliteEventStore::new(conn.clone());

    let mut sink = struct_sink::StoreSink {
        store: event_store,
        last_error: None,
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    service.ingest_request(request, capture_context, &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to pin memory: {}", err).into());
    }

    println!("Memory successfully pinned to vault.");
    Ok(())
}

fn run_safety_sync(cli: &Cli, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for ChangeGuard Hotspots...");

    let output = std::process::Command::new("changeguard")
        .args(["hotspots", "--limit", &limit.to_string()])
        .output()?;

    if !output.status.success() {
        return Err(
            "ChangeGuard scan failed. Ensure ChangeGuard is installed and initialized.".into(),
        );
    }

    let hotspots = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hotspots.is_empty() {
        println!("No hotspots identified. Safety layer is healthy.");
        return Ok(());
    }

    println!("Ingesting hotspots into AI-Brains vault...");

    let content = format!(
        "HOTSPOT: Brittle files identified by ChangeGuard:\n\n{}",
        hotspots
    );
    run_pin(
        cli,
        content,
        "assistant".to_string(),
        "LocalOnly".to_string(),
    )?;

    println!("Safety synchronization complete.");
    Ok(())
}

mod struct_sink {
    use ai_brains_capture::CaptureSink;
    use ai_brains_events::Envelope;
    use ai_brains_store::{EventStore, SqliteEventStore};

    pub struct StoreSink {
        pub store: SqliteEventStore,
        pub last_error: Option<String>,
    }
    impl CaptureSink for StoreSink {
        fn append(&mut self, envelope: Envelope) {
            if let Err(err) = self.store.append_event(&envelope) {
                self.last_error = Some(err.to_string());
            }
        }
    }
}
