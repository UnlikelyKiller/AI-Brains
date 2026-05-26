use crate::action_digest::normalize_role;
use crate::assistant_final::build_assistant_final;
use crate::errors::{CaptureError, Result, VerificationGateRejection};
use crate::git_capture::capture_metadata;
use crate::metadata::CaptureMetadata;
use crate::privacy::effective_privacy;
use crate::session_start::build_session_started;
use crate::session_stop::build_session_stop;
use crate::user_prompt::build_user_prompt;
use crate::verification_gate::{GateDecision, VerificationGate};
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
    fn set_sync_state(&mut self, _key: &str, _value: &str) {}
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

#[derive(Debug)]
pub struct CaptureService {
    /// Optional verification gate that intercepts assistant-final ingests.
    /// When `None` (default), no gating occurs — backward-compatible behaviour.
    verification_gate: Option<VerificationGate>,
}

impl Default for CaptureService {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureService {
    /// Create a service without a verification gate (backward-compatible).
    pub fn new() -> Self {
        Self {
            verification_gate: None,
        }
    }

    /// Create a service with the given verification gate enabled.
    /// The gate will intercept assistant-final ingests and may block them
    /// when ChangeGuard predicts high failure probability or detects drift.
    pub fn with_verification_gate(gate: VerificationGate) -> Self {
        Self {
            verification_gate: Some(gate),
        }
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

                // --- T43 Verification Gate ---------------------------------
                // Intercept assistant-final (ingest-final) before event
                // build.  If ChangeGuard predicts high failure probability or
                // detects ledger drift the gate blocks and returns a
                // structured error so the AI harness can self-remediate.
                if let Some(ref gate) = self.verification_gate {
                    match gate.check() {
                        GateDecision::Proceed => {
                            // All clear — continue to event build.
                        }
                        GateDecision::Blocked {
                            failure_probability,
                            drift_detected,
                            risk_level,
                            explanation,
                        } => {
                            tracing::warn!(
                                session_id = %request.session_id,
                                failure_prob = failure_probability,
                                drift = drift_detected,
                                risk = %risk_level,
                                "Verification gate BLOCKED ingest-final"
                            );
                            return Err(CaptureError::VerificationGateRejected(
                                VerificationGateRejection {
                                    session_id: request.session_id,
                                    failure_probability,
                                    drift_detected,
                                    risk_level,
                                    explanation,
                                },
                            ));
                        }
                    }
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
