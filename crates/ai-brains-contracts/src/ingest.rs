use ai_brains_core::ids::{HarnessId, SessionId, TransactionId, TurnId};
use ai_brains_core::privacy::Privacy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    pub session_id: SessionId,
    pub project_id: ai_brains_core::ids::ProjectId,
    pub harness_id: HarnessId,
    pub turn_id: TurnId,
    pub role: String,
    pub content: String,
    pub privacy: Privacy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResponse {
    pub event_id: String,
    pub processed: bool,
}
