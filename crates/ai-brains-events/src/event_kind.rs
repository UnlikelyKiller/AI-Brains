use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EventKind {
    // System
    SystemInitialized,
    RecoveryKitCreated,

    // Project
    ProjectRegistered,
    ProjectAliasAdded,

    // Session
    SessionStarted,
    UserPromptRecorded,
    AssistantFinalRecorded,
    SessionCompleted,
    SessionFailed,
    SessionSummaryCreated,

    // Memory
    MemoryPinned,
    MemoryForgotten,
    MemoryRestored,
    PrivacyEscalated,

    // Background
    NightlyJobStarted,
    ConflictDetected,
    RecipePromoted,
    MemorySynthesized,
    FeedbackMetric,

    // Catch-all for forward compatibility
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
