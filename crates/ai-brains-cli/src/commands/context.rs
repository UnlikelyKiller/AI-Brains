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
    let discovered_project_id = find_changeguard_dir(&current_dir)
        .and_then(|dir| extract_project_id_from_changeguard(&dir))
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

    Ok(())
}
