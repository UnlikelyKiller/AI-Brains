use crate::context::AppContext;
use ai_brains_store::QueryStore;

pub fn list(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let projects = ctx.conn.list_projects()?;
    println!(
        "{:<36} {:<20} {:<25} memories",
        "project_id", "name", "alias"
    );
    for (pid, name, alias, count) in projects {
        println!(
            "{:<36} {:<20} {:<25} {}",
            pid,
            &name[..std::cmp::min(20, name.len())],
            alias,
            count
        );
    }
    Ok(())
}

pub fn resolve(ctx: &AppContext, alias: &str) -> Result<(), Box<dyn std::error::Error>> {
    // First try exact alias match
    if let Some(pid) = ctx.conn.resolve_project_id_from_alias(alias)? {
        println!("{}", pid);
        return Ok(());
    }

    // Fall back to fuzzy name match
    let projects = ctx.conn.list_projects()?;
    let lower_alias = alias.to_lowercase();
    let matched: Vec<_> = projects
        .into_iter()
        .filter(|(_, name, alias_name, _)| {
            name.to_lowercase().contains(&lower_alias)
                || alias_name.to_lowercase().contains(&lower_alias)
        })
        .collect();

    if matched.len() == 1 {
        println!("{}", matched[0].0);
        Ok(())
    } else if matched.len() > 1 {
        eprintln!("Ambiguous alias '{}' — did you mean one of these?", alias);
        for (pid, name, alias_name, count) in matched {
            eprintln!("  {} | {} | {} | {} memories", pid, name, alias_name, count);
        }
        std::process::exit(1);
    } else {
        eprintln!("No project found for alias '{}'", alias);
        std::process::exit(1);
    }
}

pub fn detect(ctx: &AppContext, export_shell: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Try to detect current repo from git
    let current_dir = std::env::current_dir()?;
    let repo_slug = get_git_repo_slug(&current_dir)?;

    if let Some(slug) = repo_slug {
        // Try to resolve slug as alias or name
        let projects = ctx.conn.list_projects()?;
        let lower_slug = slug.to_lowercase();
        let matched: Vec<_> = projects
            .into_iter()
            .filter(|(_, name, alias_name, _)| {
                name.to_lowercase() == lower_slug
                    || name.to_lowercase().contains(&lower_slug)
                    || alias_name.to_lowercase() == lower_slug
                    || alias_name.to_lowercase().contains(&lower_slug)
            })
            .collect();

        if matched.len() == 1 {
            let (pid, name, alias, count) = &matched[0];
            if export_shell {
                println!("export AI_BRAINS_PROJECT_ID={}", pid);
                println!(
                    "# AI-Brains project detected: {} | alias={} | memories={}",
                    name, alias, count
                );
            } else {
                println!(
                    "Detected project from git: {} ({}) | alias={} | memories={}",
                    name, pid, alias, count
                );
            }
            return Ok(());
        } else if matched.len() > 1 {
            eprintln!(
                "Ambiguous match for '{}' — multiple candidates found in vault:",
                lower_slug
            );
            for (pid, name, alias, count) in &matched {
                eprintln!("  {} | {} | {} | {} memories", pid, name, alias, count);
            }
            if export_shell {
                eprintln!("# No unambiguous match — set AI_BRAINS_PROJECT_ID manually");
                std::process::exit(1);
            }
            return Ok(());
        }
    }

    // No match found
    if export_shell {
        eprintln!("# No project detected in current directory — run from a git repo with an AI-Brains project");
        std::process::exit(1);
    } else {
        println!("No project detected in current directory.");
    }
    Ok(())
}

fn get_git_repo_slug(path: &std::path::Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Try git rev-parse --show-toplevel
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let toplevel_path = std::path::Path::new(&toplevel);

    // Try name from directory
    if let Some(name) = toplevel_path.file_name().and_then(|n| n.to_str()) {
        let cleaned = name.to_string();
        if !cleaned.is_empty() {
            return Ok(Some(cleaned));
        }
    }

    // Try git remote
    let remote = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()?;

    if remote.status.success() {
        let url = String::from_utf8_lossy(&remote.stdout).trim().to_owned();
        // Extract repo name from git URL
        // e.g. https://github.com/user/Sneaky-Browse.git → Sneaky-Browse
        // e.g. git@github.com:user/KinLedger.git → KinLedger
        if let Some(slug) = extract_repo_name(&url) {
            return Ok(Some(slug));
        }
    }

    Ok(None)
}

fn extract_repo_name(url: &str) -> Option<String> {
    // Remove .git suffix
    let url = url.strip_suffix(".git").unwrap_or(url);

    // Match patterns:
    // https://host/path/repo.git → repo
    // git@host:user/repo.git → repo
    // ssh://host/user/repo.git → repo

    if let Some(pos) = url.rfind('/') {
        let repo = &url[pos + 1..];
        if !repo.is_empty() {
            return Some(repo.to_string());
        }
    }

    if let Some(pos) = url.rfind(':') {
        let repo = &url[pos + 1..];
        if !repo.is_empty() {
            return Some(repo.to_string());
        }
    }

    None
}
