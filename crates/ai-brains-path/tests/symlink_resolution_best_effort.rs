use ai_brains_path::normalize_project_path;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brains-path-{name}-{nanos}"))
}

#[cfg(windows)]
fn create_symlink_dir(target: &PathBuf, link: &PathBuf) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

#[cfg(unix)]
fn create_symlink_dir(target: &PathBuf, link: &PathBuf) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(not(any(windows, unix)))]
fn create_symlink_dir(_target: &PathBuf, _link: &PathBuf) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "symlinks not supported on this platform",
    ))
}

#[test]
fn symlink_resolution_best_effort() -> Result<(), Box<dyn std::error::Error>> {
    let root = unique_temp_dir("root");
    let target = root.join("target");
    let link = root.join("link");
    fs::create_dir_all(&target)?;

    let link_path = match create_symlink_dir(&target, &link) {
        Ok(_) => link,
        Err(_) => target.clone(),
    };

    let normalized = normalize_project_path(&link_path.to_string_lossy())?;
    let target_normalized = normalize_project_path(&target.to_string_lossy())?;

    assert_eq!(normalized.canonical(), target_normalized.canonical());

    let _ = fs::remove_dir_all(&root);

    Ok(())
}
