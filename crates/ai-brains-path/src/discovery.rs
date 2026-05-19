use std::path::{Path, PathBuf};

/// Searches for a .changeguard directory starting from the given path and walking up the tree.
/// Returns the path to the .changeguard directory if found.
pub fn find_changeguard_dir(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();
    loop {
        // Check .changeguard
        let changeguard_path = current.join(".changeguard");
        if changeguard_path.is_dir() {
            return Some(changeguard_path);
        }

        // Check .git/.changeguard
        let git_changeguard_path = current.join(".git").join(".changeguard");
        if git_changeguard_path.is_dir() {
            return Some(git_changeguard_path);
        }

        if !current.pop() {
            break;
        }
    }
    None
}

/// Attempts to extract a project ID from .changeguard metadata.
/// For now, we look for a 'project_id' file or entry in a config if it exists.
/// ChangeGuard often uses a fixed project ID per repo.
pub fn extract_project_id_from_changeguard(changeguard_dir: &Path) -> Option<String> {
    // Look for .changeguard/project_id
    let id_file = changeguard_dir.join("project_id");
    if id_file.exists() {
        if let Ok(content) = std::fs::read_to_string(id_file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Fallback: Check for .changeguard/config.json or similar if we knew the schema.
    // Assuming for now it's in a 'project_id' file.
    None
}
