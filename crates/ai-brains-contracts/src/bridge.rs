use ai_brains_core::ids::{ProjectId, SessionId, TransactionId};
use ai_brains_core::privacy::Privacy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRecord {
    pub bridge_version: String,
    pub direction: BridgeDirection,
    pub timestamp: String, // RFC 3339
    pub parent_hash: Option<String>,
    pub project_id: ProjectId,
    pub session_id: SessionId,
    pub tx_id: Option<TransactionId>,
    pub record_kind: String,
    pub payload: serde_json::Value,
    pub privacy: Privacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BridgeDirection {
    Inbound,
    Outbound,
}
