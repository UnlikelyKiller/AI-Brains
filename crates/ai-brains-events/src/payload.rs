use ai_brains_core::ids::{ConflictId, MemoryId, ProjectId, RecipeId, SessionId, TransactionId};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPromptRecordedPayload {
    pub session_id: SessionId,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistantFinalRecordedPayload {
    pub session_id: SessionId,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryForgottenPayload {
    pub memory_id: MemoryId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRestoredPayload {
    pub memory_id: MemoryId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummaryCreatedPayload {
    pub session_id: SessionId,
    pub project_id: Option<ProjectId>,
    pub memory_id: MemoryId,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictDetectedPayload {
    pub conflict_id: ConflictId,
    pub memory_ids: Vec<MemoryId>,
    pub session_id: SessionId,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipePromotedPayload {
    pub recipe_id: RecipeId,
    pub name: String,
    pub content: String,
    pub steps: Vec<String>,
    pub source_memory_ids: Vec<MemoryId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySynthesizedPayload {
    pub memory_id: MemoryId,
    pub level: u32,
    pub source_memory_ids: Vec<MemoryId>,
    pub project_id: ProjectId,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackMetricPayload {
    pub metric_kind: String,
    pub value: String,
    pub session_id: Option<SessionId>,
    pub project_id: Option<ProjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionRecordedPayload {
    pub session_id: SessionId,
    pub tx_id: Option<TransactionId>,
    pub predicted_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyOutcomeRecordedPayload {
    pub tx_id: TransactionId,
    pub status: String,
    pub affected_paths: Vec<String>,
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
    MemoryRestored(MemoryRestoredPayload),
    SessionSummaryCreated(SessionSummaryCreatedPayload),
    ConflictDetected(ConflictDetectedPayload),
    RecipePromoted(RecipePromotedPayload),
    MemorySynthesized(MemorySynthesizedPayload),
    FeedbackMetric(FeedbackMetricPayload),
    PredictionRecorded(PredictionRecordedPayload),
    VerifyOutcomeRecorded(VerifyOutcomeRecordedPayload),

    /// Used for unknown future events to prevent deserialization failure
    #[serde(other)]
    Unknown,
}
