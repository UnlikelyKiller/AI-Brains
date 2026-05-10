use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::errors::{AdapterError, Result};
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
/// .system_generated/logs/overview.txt.
pub fn discover_sessions() -> Result<Vec<PathBuf>> {
    let home = dirs::home_dir().ok_or(AdapterError::Other(
        "Cannot determine home directory".to_string(),
    ))?;
    let brain_dir = home.join(".gemini").join("antigravity").join("brain");

    if !brain_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    let entries = std::fs::read_dir(&brain_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let overview = path
            .join(".system_generated")
            .join("logs")
            .join("overview.txt");
        if overview.exists() {
            sessions.push(overview);
        }
    }

    Ok(sessions)
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

/// Orchestrates the import of Antigravity sessions.
pub fn import_antigravity_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    days: usize,
    project_id: ProjectId,
) -> Result<(usize, usize)> {
    let all_sessions = discover_sessions()?;
    if all_sessions.is_empty() {
        return Ok((0, 0));
    }

    let recent_sessions = filter_recent_sessions(&all_sessions, days);
    if recent_sessions.is_empty() {
        return Ok((0, 0));
    }

    // Canonical Antigravity Harness ID
    let antigravity_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| AdapterError::Other(format!("Invalid canonical harness ID: {}", e)))?;

    let mut total_turns = 0;
    let mut sessions_imported = 0;

    for overview_path in &recent_sessions {
        // Quiescence check: Skip if modified in the last 5 minutes
        if let Ok(metadata) = std::fs::metadata(overview_path) {
            if let Ok(modified) = metadata.modified() {
                if SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::ZERO)
                    < Duration::from_secs(300)
                {
                    continue;
                }
            }
        }

        let session_id_str = match session_id_from_path(overview_path) {
            Some(id) => id,
            None => continue,
        };

        let session_id = SessionId::from_uuid(
            Uuid::parse_str(&session_id_str).unwrap_or_else(|_| Uuid::new_v4()),
        );

        // Status-aware idempotency check via QueryStore
        if let Ok(Some(status)) = query_store.get_session_status(&session_id) {
            if status == "completed" {
                continue;
            }
        }

        let steps = match parse_overview_file(overview_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let turns = extract_turns(&steps);
        if turns.is_empty() {
            continue;
        }

        let capture_context = CaptureContext {
            git_working_dir: std::env::current_dir().ok(),
        };

        // Start the session
        service.start_session(
            ai_brains_capture::SessionStartCommand {
                session_id,
                project_id,
                harness_id: antigravity_harness,
                privacy: Privacy::LocalOnly,
            },
            capture_context.clone(),
            sink,
        )?;

        // Ingest each turn with deterministic Turn IDs
        for (i, turn) in turns.iter().enumerate() {
            let turn_id = TurnId::from_uuid(Uuid::new_v5(
                &session_id.as_uuid(),
                format!("turn-{}", i).as_bytes(),
            ));

            let request = IngestRequest {
                session_id,
                project_id,
                harness_id: antigravity_harness,
                turn_id,
                role: turn.role.clone(),
                content: turn.content.clone(),
                privacy: Privacy::LocalOnly,
                thinking: None,
            };
            service.ingest_request(request, capture_context.clone(), sink)?;
            total_turns += 1;
        }

        // Mark as completed
        service.stop_session(
            ai_brains_capture::SessionStopCommand {
                session_id,
                harness_id: antigravity_harness,
                privacy: Privacy::LocalOnly,
                status: SessionStopStatus::Completed,
                reason: Some("Antigravity batch import complete".to_string()),
            },
            capture_context,
            sink,
        )?;

        sessions_imported += 1;
    }

    Ok((total_turns, sessions_imported))
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
}
