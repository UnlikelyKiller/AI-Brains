use ai_brains_core::ids::{ConflictId, MemoryId, ProjectId, RecipeId, SessionId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemInitializedPayload {
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryKitCreatedPayload {
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRegisteredPayload {
    pub project_id: ProjectId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAliasAddedPayload {
    pub project_id: ProjectId,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStartedPayload {
    pub session_id: SessionId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPromptRecordedPayload {
    pub session_id: SessionId,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistantFinalRecordedPayload {
    pub session_id: SessionId,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCompletedPayload {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFailedPayload {
    pub session_id: SessionId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryPinnedPayload {
    pub memory_id: MemoryId,
    pub content: String,
    pub session_id: Option<SessionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryForgottenPayload {
    pub memory_id: MemoryId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyEscalatedPayload {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NightlyJobStartedPayload {
    pub job_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictDetectedPayload {
    pub conflict_id: ConflictId,
    pub session_id: SessionId,
    pub contradicted_memory_ids: Vec<MemoryId>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipePromotedPayload {
    pub recipe_id: RecipeId,
    pub name: String,
    pub steps: Vec<String>,
    pub source_session_ids: Vec<SessionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummaryCreatedPayload {
    pub session_id: SessionId,
    pub memory_id: MemoryId,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySynthesizedPayload {
    pub memory_id: MemoryId,
    pub content: String,
    pub source_memory_ids: Vec<MemoryId>,
    pub level: u32,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum Payload {
    SystemInitialized(SystemInitializedPayload),
    RecoveryKitCreated(RecoveryKitCreatedPayload),
    ProjectRegistered(ProjectRegisteredPayload),
    ProjectAliasAdded(ProjectAliasAddedPayload),
    SessionStarted(SessionStartedPayload),
    UserPromptRecorded(UserPromptRecordedPayload),
    AssistantFinalRecorded(AssistantFinalRecordedPayload),
    SessionCompleted(SessionCompletedPayload),
    SessionFailed(SessionFailedPayload),
    MemoryPinned(MemoryPinnedPayload),
    MemoryForgotten(MemoryForgottenPayload),
    PrivacyEscalated(PrivacyEscalatedPayload),
    NightlyJobStarted(NightlyJobStartedPayload),
    SessionSummaryCreated(SessionSummaryCreatedPayload),
    ConflictDetected(ConflictDetectedPayload),
    RecipePromoted(RecipePromotedPayload),
    MemorySynthesized(MemorySynthesizedPayload),

    /// Used for unknown future events to prevent deserialization failure
    #[serde(other)]
    Unknown,
}
