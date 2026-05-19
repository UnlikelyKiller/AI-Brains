use crate::errors::{PathError, Result};

pub fn is_wsl_mount_path(input: &str) -> bool {
    input.to_lowercase().starts_with("/mnt/")
}

pub fn wsl_to_windows(input: &str) -> Result<String> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();
    let rest = if lower.starts_with("/mnt/") {
        &trimmed[5..]
    } else {
        return Err(PathError::MalformedWslPath(trimmed.to_string()));
    };

    let mut parts = rest.split('/');
    let drive = parts
        .next()
        .ok_or_else(|| PathError::MalformedWslPath(trimmed.to_string()))?;

    if drive.len() != 1 {
        return Err(PathError::MalformedWslPath(trimmed.to_string()));
    }

    let drive_char = drive
        .chars()
        .next()
        .ok_or_else(|| PathError::MalformedWslPath(trimmed.to_string()))?;

    if !drive_char.is_ascii_alphabetic() {
        return Err(PathError::MalformedWslPath(trimmed.to_string()));
    }

    let mut windows = format!("{}:\\", drive_char.to_ascii_uppercase());
    let remainder = parts.collect::<Vec<_>>().join("\\");
    if !remainder.is_empty() {
        windows.push_str(&remainder);
    }

    Ok(windows)
}
