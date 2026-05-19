use crate::action_digest::normalize_role;
use crate::assistant_final::build_assistant_final;
use crate::errors::{CaptureError, Result};
use crate::git_capture::capture_metadata;
use crate::metadata::CaptureMetadata;
use crate::privacy::effective_privacy;
use crate::session_start::build_session_started;
use crate::session_stop::build_session_stop;
use crate::user_prompt::build_user_prompt;
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TransactionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::Envelope;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct CaptureContext {
    pub git_working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct SessionStartCommand {
    pub session_id: SessionId,
    pub project_id: ProjectId,
    pub harness_id: HarnessId,
    pub privacy: Privacy,
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStopStatus {
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct SessionStopCommand {
    pub session_id: SessionId,
    pub harness_id: HarnessId,
    pub privacy: Privacy,
    pub status: SessionStopStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CaptureOutcome {
    pub events: Vec<Envelope>,
    pub effective_privacy: Privacy,
    pub metadata: CaptureMetadata,
}

pub trait CaptureSink {
    fn append(&mut self, envelope: Envelope);
}

#[derive(Debug, Default)]
pub struct MemorySink {
    events: Vec<Envelope>,
}

impl MemorySink {
    pub fn into_events(self) -> Vec<Envelope> {
        self.events
    }
}

impl CaptureSink for MemorySink {
    fn append(&mut self, envelope: Envelope) {
        self.events.push(envelope);
    }
}

#[derive(Debug, Default)]
pub struct CaptureService;

impl CaptureService {
    pub fn new() -> Self {
        Self
    }

    pub fn start_session(
        &self,
        command: SessionStartCommand,
        context: CaptureContext,
        sink: &mut impl CaptureSink,
    ) -> Result<CaptureOutcome> {
        let metadata = capture_metadata(&context)?;
        let event = build_session_started(&command)?;
        sink.append(event.clone());
        Ok(CaptureOutcome {
            events: vec![event],
            effective_privacy: command.privacy,
            metadata,
        })
    }

    pub fn ingest_request(
        &self,
        request: IngestRequest,
        context: CaptureContext,
        sink: &mut impl CaptureSink,
    ) -> Result<CaptureOutcome> {
        let role = normalize_role(&request.role);
        let content = request.content.trim();

        let effective_privacy = effective_privacy(&request.content, request.privacy);
        let metadata = capture_metadata(&context)?;

        let event = match role.as_str() {
            "user" => {
                if content.is_empty() {
                    return Err(CaptureError::EmptyPrompt);
                }
                build_user_prompt(&request, effective_privacy)?
            }
            "assistant" => {
                if content.is_empty() {
                    return Err(CaptureError::EmptyFinal);
                }
                build_assistant_final(&request, effective_privacy)?
            }
            _ => return Err(CaptureError::UnsupportedRole(request.role)),
        };

        sink.append(event.clone());
        Ok(CaptureOutcome {
            events: vec![event],
            effective_privacy,
            metadata,
        })
    }

    pub fn stop_session(
        &self,
        command: SessionStopCommand,
        context: CaptureContext,
        sink: &mut impl CaptureSink,
    ) -> Result<CaptureOutcome> {
        let metadata = capture_metadata(&context)?;
        let event = build_session_stop(&command)?;
        sink.append(event.clone());
        Ok(CaptureOutcome {
            events: vec![event],
            effective_privacy: command.privacy,
            metadata,
        })
    }
}
