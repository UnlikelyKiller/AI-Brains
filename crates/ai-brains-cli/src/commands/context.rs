use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_path::{extract_project_id_from_changeguard, find_changeguard_dir};

pub fn run(
    new_project: bool,
    new_session: bool,
    show: bool,
    tx_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project");

    let env_path = current_dir.join(".env");

    if show {
        if env_path.exists() {
            let content = std::fs::read_to_string(&env_path)?;
            println!("--- Current Context ---");
            for line in content.lines() {
                if line.starts_with("AI_BRAINS_") {
                    println!("{}", line);
                }
            }
            println!("Repository: {}", current_dir.display());
        } else {
            println!(
                "No .env file found in {}. Run 'ai-brains context' to initialize.",
                current_dir.display()
            );
        }
        return Ok(());
    }

    // Auto-discovery from ChangeGuard
    let changeguard_dir = find_changeguard_dir(&current_dir);
    let discovered_project_id = changeguard_dir
        .as_ref()
        .and_then(|dir| extract_project_id_from_changeguard(dir))
        .and_then(|id_str| id_str.parse::<ProjectId>().ok());

    let project_id = if new_project {
        ProjectId::new()
    } else if let Some(id) = discovered_project_id {
        println!("Auto-discovered project ID from .changeguard: {}", id);
        id
    } else {
        // Deterministic project ID based on the canonical directory path
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&current_dir.to_string_lossy().to_lowercase(), &mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&hash.to_be_bytes());
        ProjectId::from_uuid(uuid::Uuid::from_bytes(bytes))
    };

    // Check for existing session
    let existing_session = if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path)?;
        existing
            .lines()
            .find(|l| l.starts_with("AI_BRAINS_SESSION_ID"))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.to_string())
    } else {
        None
    };

    if let Some(ref sid) = existing_session {
        if !new_session {
            eprintln!(
                "Session {} already exists. Use --new-session to replace, or --show to view.",
                sid
            );
            return Err("session already exists".into());
        }
        println!("Replacing existing session: {}", sid);
    }

    let session_id = SessionId::new();
    let harness_id = HarnessId::new();

    let mut env_content = format!(
        "AI_BRAINS_PROJECT_ID={}\nAI_BRAINS_SESSION_ID={}\nAI_BRAINS_HARNESS_ID={}\n",
        project_id, session_id, harness_id
    );

    if let Some(id) = tx_id {
        env_content.push_str(&format!("CHANGEGUARD_TX_ID={}\n", id));
    }

    let mut final_content = if env_path.exists() {
        let existing = std::fs::read_to_string(&env_path)?;
        existing
            .lines()
            .filter(|l| {
                !l.starts_with("AI_BRAINS_PROJECT_ID")
                    && !l.starts_with("AI_BRAINS_SESSION_ID")
                    && !l.starts_with("AI_BRAINS_HARNESS_ID")
                    && !l.starts_with("CHANGEGUARD_TX_ID")
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

    // Auto-trigger sync pull (only when .changeguard is discovered in the repo)
    if changeguard_dir.is_some() {
        // 1. Load the newly updated local .env
        if env_path.exists() {
            dotenvy::from_path_override(&env_path).ok();
        }

        // 2. Resolve vault path and key from environment or fallback
        let vault_path = std::env::var("AI_BRAINS_VAULT_PATH")
            .map(std::path::PathBuf::from)
            .or_else(|_| {
                // Check global env
                if let Some(mut home) = dirs::home_dir() {
                    home.push(".ai-brains");
                    home.push(".env");
                    if home.exists() {
                        dotenvy::from_path_override(&home).ok();
                    }
                }
                std::env::var("AI_BRAINS_VAULT_PATH").map(std::path::PathBuf::from)
            })
            .unwrap_or_else(|_| {
                let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                path.push(".ai-brains");
                path.push("vault.db");
                path
            });

        let vault_key = std::env::var("AI_BRAINS_VAULT_KEY").ok();

        // 3. Create context and execute sync pull (best-effort)
        println!("Auto-triggering sync pull from ChangeGuard...");
        match crate::context::AppContext::from_cli(Some(vault_path), vault_key) {
            Ok(ctx) => {
                if let Err(e) = crate::commands::sync::run_pull(&ctx, None, false, false) {
                    eprintln!("Auto-trigger sync pull failed: {}", e);
                } else {
                    println!("Auto-trigger sync pull completed successfully.");
                }
            }
            Err(e) => {
                eprintln!("Auto-trigger sync pull skipped (vault unavailable): {}", e);
            }
        }
    }

    Ok(())
}
