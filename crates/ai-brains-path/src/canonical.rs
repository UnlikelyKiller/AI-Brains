use crate::alias::resolve_alias;
use crate::display::display_path;
use crate::errors::{PathError, Result};
use crate::project_path::ProjectPath;
use crate::symlink::resolve_best_effort;
use crate::unc::{is_unc_path, normalize_unc};
use crate::windows::{has_drive_prefix, normalize_drive_path, strip_extended_length_prefix};
use crate::wsl::{is_wsl_mount_path, wsl_to_windows};

pub fn normalize_project_path(input: &str) -> Result<ProjectPath> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PathError::EmptyInput);
    }

    if trimmed.contains('\0') {
        return Err(PathError::NulByte);
    }

    let display = display_path(trimmed);
    let aliased = resolve_alias(trimmed);
    let normalized_input = if is_wsl_mount_path(&aliased) {
        wsl_to_windows(&aliased)?
    } else {
        strip_extended_length_prefix(&aliased)
    };

    let canonical = if is_unc_path(&normalized_input) {
        normalize_unc(&normalized_input)
    } else if has_drive_prefix(&normalized_input) {
        normalize_drive_path(&normalized_input)?
    } else {
        // Resolve relative path
        let mut abs = std::env::current_dir().map_err(|e| PathError::IoError(e.to_string()))?;
        abs.push(trimmed);
        let abs_str = abs.to_string_lossy().to_string();
        if is_unc_path(&abs_str) {
            normalize_unc(&abs_str)
        } else if has_drive_prefix(&abs_str) {
            normalize_drive_path(&abs_str)?
        } else if abs_str.starts_with('/') {
            abs_str
        } else {
            return Err(PathError::RelativePath(trimmed.to_string()));
        }
    };

    let canonical = resolve_best_effort(&canonical);
    Ok(ProjectPath::new(canonical, display))
}
