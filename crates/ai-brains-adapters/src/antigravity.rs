use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::errors::Result;
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

pub fn antigravity_capability() -> AdapterCapability {
    AdapterCapability {
        name: "antigravity".to_string(),
        level: CapabilityLevel::Partial,
        supports_hooks: false,
        supports_wrapper_mode: false,
        notes: "Batch import via nightly or antigravity-import command. No real-time hooks."
            .to_string(),
    }
}

pub fn manual_import_instructions() -> String {
    "Antigravity sessions are auto-imported by `ai-brains nightly` or `ai-brains antigravity-import`. Manual pinning is still recommended for decisions during the session.".to_string()
}

/// A single step from an Antigravity overview.txt JSONL file.
#[derive(Debug, Clone, Deserialize)]
pub struct AntigravityStep {
    pub step_index: u32,
    pub source: String,
    #[serde(rename = "type")]
    pub step_type: String,
    pub content: Option<String>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
}

/// A conversation turn extracted from Antigravity logs, ready for ingestion.
#[derive(Debug, Clone)]
pub struct AntigravityTurn {
    pub role: String,
    pub content: String,
    pub created_at: Option<String>,
}

/// Discover Antigravity brain directories containing overview.txt files.
/// Scans ~/.gemini/antigravity/brain/ for subdirectories with
/// .system_generated/logs/overview.txt. Also scans WSL paths.
pub fn discover_sessions() -> Result<Vec<AntigravitySessionSource>> {
    let mut all_sources = Vec::new();

    // Windows-side Home Dir
    if let Some(home) = dirs::home_dir() {
        let gemini_base = home.join(".gemini");

        // 1. Tool Brain Directories
        let tool_dirs = vec!["antigravity", "antigravity-cli", "antigravity-ide"];
        for tool in tool_dirs {
            let brain_path = gemini_base.join(tool).join("brain");
            if brain_path.exists() {
                scan_brain_dir(&brain_path, &mut all_sources)?;
            }
        }

        // 2. Project Temp Directories
        let tmp_path = gemini_base.join("tmp");
        if tmp_path.exists() {
            scan_tmp_dirs(&tmp_path, &mut all_sources)?;
        }
    }

    // WSL-side Antigravity (Ubuntu) - Legacy Support
    let wsl_brain = PathBuf::from(r"\\wsl$\Ubuntu\home\ryan\.gemini\antigravity\brain");
    if wsl_brain.exists() {
        scan_brain_dir(&wsl_brain, &mut all_sources)?;
    }

    Ok(all_sources)
}

#[derive(Debug, Clone)]
pub enum AntigravityFormat {
    BrainLog,    // overview.txt or transcript.jsonl
    ProjectChat, // session-*.jsonl
}

#[derive(Debug, Clone)]
pub struct AntigravitySessionSource {
    pub path: PathBuf,
    pub session_id: String,
    pub format: AntigravityFormat,
    pub project_hash: Option<String>,
}

fn scan_brain_dir(brain_dir: &Path, sources: &mut Vec<AntigravitySessionSource>) -> Result<()> {
    let entries = std::fs::read_dir(brain_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let session_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Check for overview.txt (legacy)
        let overview = path
            .join(".system_generated")
            .join("logs")
            .join("overview.txt");
        if overview.exists() {
            sources.push(AntigravitySessionSource {
                path: overview,
                session_id: session_id.clone(),
                format: AntigravityFormat::BrainLog,
                project_hash: None,
            });
        }

        // Check for transcript.jsonl (new agy)
        let transcript = path
            .join(".system_generated")
            .join("logs")
            .join("transcript.jsonl");
        if transcript.exists() {
            sources.push(AntigravitySessionSource {
                path: transcript,
                session_id,
                format: AntigravityFormat::BrainLog,
                project_hash: None,
            });
        }
    }
    Ok(())
}

fn scan_tmp_dirs(tmp_base: &Path, sources: &mut Vec<AntigravitySessionSource>) -> Result<()> {
    let project_entries = std::fs::read_dir(tmp_base)?;
    for project_entry in project_entries {
        let project_entry = project_entry?;
        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let project_hash = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        let chats_dir = project_path.join("chats");
        if chats_dir.exists() {
            let chat_entries = std::fs::read_dir(chats_dir)?;
            for chat_entry in chat_entries {
                let chat_entry = chat_entry?;
                let chat_path = chat_entry.path();
                if chat_path.is_file() && chat_path.extension().is_some_and(|ext| ext == "jsonl") {
                    let session_id = chat_path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .map(|s| s.replace("session-", ""))
                        .unwrap_or_default();

                    sources.push(AntigravitySessionSource {
                        path: chat_path,
                        session_id,
                        format: AntigravityFormat::ProjectChat,
                        project_hash: project_hash.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Filter sessions to those modified within the last N days.
pub fn filter_recent_sessions(paths: &[PathBuf], days: usize) -> Vec<PathBuf> {
    let cutoff =
        std::time::SystemTime::now() - std::time::Duration::from_secs(days as u64 * 24 * 60 * 60);

    paths
        .iter()
        .filter(|p| {
            if let Ok(metadata) = std::fs::metadata(p) {
                if let Ok(modified) = metadata.modified() {
                    return modified >= cutoff;
                }
            }
            false
        })
        .cloned()
        .collect()
}

/// Extract the conversation ID (directory name) from an overview.txt path.
pub fn session_id_from_path(path: &Path) -> Option<String> {
    // Path: .../brain/<conversation-id>/.system_generated/logs/overview.txt
    path.parent() // logs/
        .and_then(|p| p.parent()) // .system_generated/
        .and_then(|p| p.parent()) // <conversation-id>/
        .and_then(|p| p.file_name())
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

/// Parse an Antigravity overview.txt JSONL file into steps.
pub fn parse_overview_file(path: &Path) -> Result<Vec<AntigravityStep>> {
    let content = std::fs::read_to_string(path)?;
    let mut steps = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<AntigravityStep>(line) {
            Ok(step) => steps.push(step),
            Err(_) => continue, // Skip malformed lines
        }
    }

    Ok(steps)
}

/// Extract ingestable turns from Antigravity steps, applying mandate #4
/// (no hidden thinking or tool logs).
///
/// Rules:
/// - USER_EXPLICIT / USER_INPUT -> role "user", strip XML metadata tags
/// - MODEL / PLANNER_RESPONSE with non-empty content -> role "assistant"
/// - MODEL / PLANNER_RESPONSE with only tool_calls (no content) -> skip
/// - MODEL / TOOL_OUTPUT -> skip
pub fn extract_turns(steps: &[AntigravityStep]) -> Vec<AntigravityTurn> {
    let mut turns = Vec::new();

    for step in steps {
        match (step.source.as_str(), step.step_type.as_str()) {
            ("USER_EXPLICIT", "USER_INPUT") => {
                if let Some(content) = &step.content {
                    let cleaned = strip_user_xml_tags(content);
                    if !cleaned.is_empty() {
                        turns.push(AntigravityTurn {
                            role: "user".to_string(),
                            content: cleaned,
                            created_at: step.created_at.clone(),
                        });
                    }
                }
            }
            ("MODEL", "PLANNER_RESPONSE") => {
                // Only keep responses that have text content (not tool-only calls)
                if let Some(content) = &step.content {
                    if !content.trim().is_empty() {
                        turns.push(AntigravityTurn {
                            role: "assistant".to_string(),
                            content: content.clone(),
                            created_at: step.created_at.clone(),
                        });
                    }
                }
                // Skip tool_calls-only responses (mandate #4)
            }
            // Skip TOOL_OUTPUT, and any other types
            _ => {}
        }
    }

    turns
}

/// Strip Antigravity XML metadata tags from user input content.
/// Removes: <USER_REQUEST>, <ADDITIONAL_METADATA>, <USER_SETTINGS_CHANGE>
/// and their closing tags, keeping only the user's actual prompt text.
pub fn strip_user_xml_tags(content: &str) -> String {
    let mut result = content.to_string();

    // Remove <ADDITIONAL_METADATA>...</ADDITIONAL_METADATA>
    result = strip_xml_block(&result, "ADDITIONAL_METADATA");

    // Remove <USER_SETTINGS_CHANGE>...</USER_SETTINGS_CHANGE>
    result = strip_xml_block(&result, "USER_SETTINGS_CHANGE");

    // Extract content from <USER_REQUEST>...</USER_REQUEST> (keep inner text)
    if let Some(extracted) = extract_xml_content(&result, "USER_REQUEST") {
        result = extracted;
    }

    result.trim().to_string()
}

fn strip_xml_block(content: &str, tag: &str) -> String {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    let mut result = String::new();
    let mut remaining = content;
    let mut in_block = false;
    let mut depth = 0;

    while let Some(pos) = if in_block {
        remaining.find(&close).or_else(|| remaining.find(&open))
    } else {
        remaining.find(&open)
    } {
        if !in_block {
            result.push_str(&remaining[..pos]);
            remaining = &remaining[pos + open.len()..];
            in_block = true;
            depth = 1;
        } else if remaining[pos..].starts_with(&close) {
            depth -= 1;
            remaining = &remaining[pos + close.len()..];
            if depth == 0 {
                in_block = false;
            }
        } else {
            depth += 1;
            remaining = &remaining[pos + open.len()..];
        }
    }

    if !in_block {
        result.push_str(remaining);
    }

    result
}

fn extract_xml_content(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    let start = content.find(&open)?;
    let end = content.find(&close)?;
    if end <= start {
        return None;
    }

    Some(content[start + open.len()..end].trim().to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChatTurn {
    #[allow(dead_code)]
    pub id: String,
    #[serde(rename = "type")]
    pub turn_type: String, // e.g. "user", "gemini", "claude"
    pub content: String,
    #[allow(dead_code)]
    pub thoughts: Option<String>,
}

/// Parse a project-specific session-*.jsonl file.
pub fn parse_project_chat_file(path: &Path) -> Result<Vec<AntigravityTurn>> {
    let content = std::fs::read_to_string(path)?;
    let mut turns = Vec::new();

    let mut lines = content.lines();
    // Skip header line
    let _header = lines.next();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(turn) = serde_json::from_str::<ProjectChatTurn>(line) {
            // Capture Privacy Mandate: Only capture user and known model roles.
            // Ignore tool_output, system, and other internal events.
            let role = match turn.turn_type.as_str() {
                "user" => Some("user"),
                "gemini" | "claude" | "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-5.3-codex"
                | "gpt-5.5-thinking" => Some("assistant"),
                _ => None,
            };

            if let Some(role) = role {
                // Apply mandate #4 (log only final assistant response, skip internal thoughts)
                if !turn.content.trim().is_empty() {
                    turns.push(AntigravityTurn {
                        role: role.to_string(),
                        content: turn.content,
                        created_at: None, // Timestamps are in header or not easily per-turn
                    });
                }
            }
        }
    }

    Ok(turns)
}

/// Orchestrates the import of Antigravity sessions from all discovered locations.
#[allow(clippy::disallowed_methods)]
pub fn import_antigravity_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    days: usize,
    default_project_id: ProjectId,
) -> Result<(usize, usize)> {
    let all_sources = discover_sessions()?;
    if all_sources.is_empty() {
        return Ok((0, 0));
    }

    // Filter by recency (optional, but good for performance)
    let recent_sources: Vec<AntigravitySessionSource> = all_sources
        .into_iter()
        .filter(|s| {
            if let Ok(metadata) = std::fs::metadata(&s.path) {
                if let Ok(modified) = metadata.modified() {
                    let cutoff =
                        SystemTime::now() - Duration::from_secs(days as u64 * 24 * 60 * 60);
                    return modified >= cutoff;
                }
            }
            false
        })
        .collect();

    let source_count = recent_sources.len();
    if source_count == 0 {
        return Ok((0, 0));
    }
    eprintln!(
        "[Antigravity] Found {} sessions modified in the last {} days. Scanning for new turns...",
        source_count, days
    );

    // Canonical Antigravity Harness IDs
    let antigravity_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| crate::errors::AdapterError::Other(format!("Invalid static ID: {}", e)))?;
    let agy_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000002")
        .map_err(|e| crate::errors::AdapterError::Other(format!("Invalid static ID: {}", e)))?;

    let mut total_turns = 0;
    let mut sessions_imported = 0;

    for (idx, source) in recent_sources.iter().enumerate() {
        // Lightweight metadata check to avoid heavy parsing of unchanged files
        let metadata = std::fs::metadata(&source.path).ok();
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

        let meta_key = format!("source_meta:{}", source.session_id);
        let stored_meta = query_store.get_sync_state(&meta_key).unwrap_or(None);
        let current_meta = format!("{}:{}", mtime, size);

        if stored_meta.as_ref() == Some(&current_meta) {
            // Already fully ingested and hasn't changed since.
            continue;
        }

        if (idx + 1) % 10 == 0 || idx == 0 || idx == source_count - 1 {
            eprintln!(
                "[Antigravity] Scanning session {}/{}...",
                idx + 1,
                source_count
            );
        }

        // Quiescence check: Skip if modified in the last 5 minutes (still active)
        if let Some(modified) = metadata.as_ref().and_then(|m| m.modified().ok()) {
            if SystemTime::now()
                .duration_since(modified)
                .unwrap_or(Duration::ZERO)
                < Duration::from_secs(300)
            {
                continue;
            }
        }

        let session_id = SessionId::from_uuid(
            Uuid::parse_str(&source.session_id).unwrap_or_else(|_| Uuid::new_v4()),
        );

        let (turns, harness_id) = match source.format {
            AntigravityFormat::BrainLog => {
                let steps = parse_overview_file(&source.path)?;
                (extract_turns(&steps), antigravity_harness)
            }
            AntigravityFormat::ProjectChat => (parse_project_chat_file(&source.path)?, agy_harness),
        };

        if turns.is_empty() {
            // Update metadata anyway so we don't keep re-parsing empty/tool-only sessions
            update_source_meta(sink, &meta_key, &current_meta);
            continue;
        }

        // Delta Sync: Check turn count in vault instead of session status (Requirement T49.1)
        let max_turn = query_store.get_max_turn_index(&session_id).unwrap_or(None);
        let next_index = max_turn.map(|m| m + 1).unwrap_or(0);

        if turns.len() <= next_index as usize {
            // No new turns, but file might have changed (e.g. metadata or tool logs).
            // Update stored metadata to reflect current state so we skip next time.
            update_source_meta(sink, &meta_key, &current_meta);
            continue;
        }

        // Mapping: Use projectHash from source if available, else resolve/default.
        let project_id = if let Some(ref hash) = source.project_hash {
            query_store
                .resolve_project_id_from_alias(hash)
                .unwrap_or(None)
                .unwrap_or(default_project_id)
        } else {
            default_project_id
        };

        let capture_context = CaptureContext {
            git_working_dir: std::env::current_dir().ok(),
        };

        // Start the session
        service.start_session(
            ai_brains_capture::SessionStartCommand {
                session_id,
                project_id,
                harness_id,
                privacy: Privacy::LocalOnly,
                tx_id: None,
            },
            capture_context.clone(),
            sink,
        )?;

        // Ingest new turns only (Delta Sync)
        for (i, turn) in turns.iter().enumerate().skip(next_index as usize) {
            let turn_id = TurnId::from_uuid(Uuid::new_v5(
                &session_id.as_uuid(),
                format!("turn-{}", i).as_bytes(),
            ));

            let request = IngestRequest {
                session_id,
                project_id,
                harness_id,
                turn_id,
                role: turn.role.clone(),
                content: turn.content.clone(),
                privacy: Privacy::LocalOnly,
                thinking: None,
                tx_id: None,
            };
            service.ingest_request(request, capture_context.clone(), sink)?;
            total_turns += 1;
        }

        // Mark as completed
        service.stop_session(
            ai_brains_capture::SessionStopCommand {
                session_id,
                harness_id,
                privacy: Privacy::LocalOnly,
                status: SessionStopStatus::Completed,
                reason: Some("Antigravity multi-path import complete".to_string()),
            },
            capture_context,
            sink,
        )?;

        // Successfully imported all current turns. Store metadata.
        update_source_meta(sink, &meta_key, &current_meta);
        sessions_imported += 1;
    }

    Ok((total_turns, sessions_imported))
}

fn update_source_meta<S: CaptureSink>(sink: &mut S, key: &str, value: &str) {
    // We try to downcast the sink to something that can handle sync state.
    // In our CLI implementation, this is StoreSink which has access to SqliteEventStore.
    // If the sink doesn't support it, we just skip it (performance degrades to legacy behavior).
    sink.set_sync_state(key, value);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]

    use super::*;

    #[test]
    fn extract_turns_keeps_user_and_assistant_content() {
        let steps = vec![
            AntigravityStep {
                step_index: 0,
                source: "USER_EXPLICIT".to_string(),
                step_type: "USER_INPUT".to_string(),
                content: Some("<USER_REQUEST>\nhello\n</USER_REQUEST>".to_string()),
                created_at: Some("2026-05-01T00:00:00Z".to_string()),
                tool_calls: vec![],
            },
            AntigravityStep {
                step_index: 4,
                source: "MODEL".to_string(),
                step_type: "PLANNER_RESPONSE".to_string(),
                content: Some("Here is the answer.".to_string()),
                created_at: Some("2026-05-01T00:00:01Z".to_string()),
                tool_calls: vec![],
            },
        ];

        let turns = extract_turns(&steps);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "Here is the answer.");
    }

    #[test]
    fn extract_turns_skips_tool_only_responses() {
        let steps = vec![
            AntigravityStep {
                step_index: 0,
                source: "USER_EXPLICIT".to_string(),
                step_type: "USER_INPUT".to_string(),
                content: Some("<USER_REQUEST>\nread the file\n</USER_REQUEST>".to_string()),
                created_at: None,
                tool_calls: vec![],
            },
            AntigravityStep {
                step_index: 4,
                source: "MODEL".to_string(),
                step_type: "PLANNER_RESPONSE".to_string(),
                content: None, // Tool call only, no text content
                created_at: None,
                tool_calls: vec![
                    serde_json::from_str(r#"{"name": "view_file"}"#).expect("valid json")
                ],
            },
            AntigravityStep {
                step_index: 8,
                source: "MODEL".to_string(),
                step_type: "PLANNER_RESPONSE".to_string(),
                content: Some("The file contains...".to_string()),
                created_at: None,
                tool_calls: vec![],
            },
        ];

        let turns = extract_turns(&steps);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[1].role, "assistant");
    }

    #[test]
    fn extract_turns_skips_tool_output() {
        let steps = vec![AntigravityStep {
            step_index: 5,
            source: "MODEL".to_string(),
            step_type: "TOOL_OUTPUT".to_string(),
            content: Some("file contents here".to_string()),
            created_at: None,
            tool_calls: vec![],
        }];

        let turns = extract_turns(&steps);
        assert!(turns.is_empty());
    }

    #[test]
    fn strip_xml_tags_extracts_user_request() {
        let input = "<USER_REQUEST>\ndo the ai brains preflight\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nThe current local time is: 2026-05-01\n</ADDITIONAL_METADATA>";
        let result = strip_user_xml_tags(input);
        assert_eq!(result, "do the ai brains preflight");
        assert!(!result.contains("ADDITIONAL_METADATA"));
    }

    #[test]
    fn strip_xml_tags_handles_no_tags() {
        let input = "plain text with no tags";
        assert_eq!(strip_user_xml_tags(input), "plain text with no tags");
    }

    #[test]
    fn strip_xml_tags_removes_settings_change() {
        let input = "<USER_REQUEST>\nhello\n</USER_REQUEST>\n<USER_SETTINGS_CHANGE>\nModel changed\n</USER_SETTINGS_CHANGE>";
        let result = strip_user_xml_tags(input);
        assert_eq!(result, "hello");
        assert!(!result.contains("SETTINGS_CHANGE"));
    }

    #[test]
    fn session_id_from_path_extracts_uuid() {
        let path = PathBuf::from(
            "C:/Users/RyanB/.gemini/antigravity/brain/26c85130-1a0b-4832-bb88-6cdd68d5f4ad/.system_generated/logs/overview.txt",
        );
        let id = session_id_from_path(&path);
        assert_eq!(id, Some("26c85130-1a0b-4832-bb88-6cdd68d5f4ad".to_string()));
    }

    #[test]
    fn parse_overview_file_handles_empty() {
        let dir = std::env::temp_dir().join("ai-brains-test-overview");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("overview.txt");
        let _ = std::fs::write(&path, "");

        let steps = parse_overview_file(&path).expect("parse should succeed");
        assert!(steps.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_overview_file_parses_jsonl() {
        let dir = std::env::temp_dir().join("ai-brains-test-overview-jsonl");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("overview.txt");

        let line1 = r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-05-01T00:00:00Z","content":"<USER_REQUEST>\nhello\n</USER_REQUEST>","tool_calls":[]}"#;
        let line2 = r#"{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-05-01T00:00:01Z","content":"Hi there","tool_calls":[]}"#;
        let line3 = r#"{"step_index":8,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-05-01T00:00:02Z","tool_calls":[{"name":"view_file"}]}"#;

        let _ = std::fs::write(&path, format!("{line1}\n{line2}\n{line3}\n"));

        let steps = parse_overview_file(&path).expect("parse should succeed");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].source, "USER_EXPLICIT");
        assert_eq!(steps[1].content.as_deref(), Some("Hi there"));
        assert!(steps[2].content.is_none());

        let turns = extract_turns(&steps);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "Hi there");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_project_chat_file_parses_jsonl() {
        let dir = std::env::temp_dir().join("ai-brains-test-project-chat");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("session-abc.jsonl");

        let header = r#"{"sessionId":"abc","projectHash":"xyz"}"#;
        let line1 = r#"{"id":"1","type":"user","content":"hello"}"#;
        let line2 = r#"{"id":"2","type":"gemini","content":"hi","thoughts":"planning..."}"#;
        let line3 = r#"{"id":"3","type":"tool_output","content":"ls output","thoughts":""}"#;

        let _ = std::fs::write(&path, format!("{header}\n{line1}\n{line2}\n{line3}\n"));

        let turns = parse_project_chat_file(&path).expect("parse should succeed");
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "hi");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
