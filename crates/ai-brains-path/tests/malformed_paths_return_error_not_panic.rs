use ai_brains_path::{normalize_project_path, PathError};

#[test]
fn relative_paths_now_supported() -> Result<(), Box<dyn std::error::Error>> {
    let relative = normalize_project_path("relative/path")?;
    // Canonical path is absolute and resolved against current_dir;
    // it should contain the relative components, regardless of separator style.
    let canonical = relative.canonical();
    assert!(
        canonical.contains("relative") && canonical.contains("path"),
        "Canonical path should contain relative path components: {}",
        canonical
    );
    assert_eq!(relative.display(), "relative/path");
    Ok(())
}

#[test]
fn malformed_paths_return_error_not_panic() {
    let malformed_wsl = normalize_project_path("/mnt/1/project");
    assert_eq!(
        malformed_wsl,
        Err(PathError::MalformedWslPath("/mnt/1/project".to_string()))
    );
}
