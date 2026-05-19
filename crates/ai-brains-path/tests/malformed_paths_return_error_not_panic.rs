use ai_brains_path::{normalize_project_path, PathError};

#[test]
fn relative_paths_now_supported() -> Result<(), Box<dyn std::error::Error>> {
    let relative = normalize_project_path("relative/path")?;
    assert!(relative.canonical().ends_with("relative\\path"));
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
