#![allow(dead_code)]

use ai_brains_capture::{CaptureContext, CaptureService, MemorySink};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn service() -> CaptureService {
    CaptureService::new()
}

pub fn sink() -> MemorySink {
    MemorySink::default()
}

pub fn ingest_request(role: &str, content: &str) -> IngestRequest {
    IngestRequest {
        session_id: SessionId::new(),
        project_id: ProjectId::new(),
        harness_id: HarnessId::new(),
        turn_id: TurnId::new(),
        role: role.to_string(),
        content: content.to_string(),
        privacy: Privacy::CloudOk,
        thinking: None,
        tx_id: None,
    }
}

pub fn context() -> CaptureContext {
    CaptureContext::default()
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    std::env::temp_dir().join(format!("ai-brains-capture-{name}-{nanos}"))
}

fn run_git(path: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("git").args(args).current_dir(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {} failed", args.join(" ")).into())
    }
}

pub fn git_context() -> Result<CaptureContext, Box<dyn std::error::Error>> {
    let root = unique_temp_dir("git");
    fs::create_dir_all(&root)?;
    run_git(&root, &["init"])?;
    run_git(&root, &["config", "user.name", "AI Brains Test"])?;
    run_git(&root, &["config", "user.email", "tests@example.com"])?;
    fs::write(root.join("README.md"), "hello\n")?;
    run_git(&root, &["add", "."])?;
    run_git(&root, &["commit", "-m", "initial"])?;
    Ok(CaptureContext {
        git_working_dir: Some(root),
    })
}

pub fn start_command() -> ai_brains_capture::SessionStartCommand {
    ai_brains_capture::SessionStartCommand {
        session_id: SessionId::new(),
        project_id: ProjectId::new(),
        harness_id: HarnessId::new(),
        privacy: Privacy::CloudOk,
        tx_id: None,
    }
}

pub fn stop_command(
    status: ai_brains_capture::SessionStopStatus,
) -> ai_brains_capture::SessionStopCommand {
    ai_brains_capture::SessionStopCommand {
        session_id: SessionId::new(),
        harness_id: HarnessId::new(),
        privacy: Privacy::CloudOk,
        status,
        reason: Some("failure".to_string()),
    }
}
