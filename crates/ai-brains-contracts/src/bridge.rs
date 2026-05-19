use ai_brains_core::privacy::Privacy;
use serde::{Deserialize, Serialize};

/// Bridge interchange record. Uses flexible string types for cross-repo compatibility.
/// Conversion to typed IDs happens at the ingest boundary, not at deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRecord {
    pub bridge_version: String,
    pub direction: BridgeDirection,
    pub timestamp: String, // RFC 3339
    pub parent_hash: Option<String>,
    pub project_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub tx_id: Option<String>,
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

impl BridgeRecord {
    pub fn formatted_payload(&self) -> String {
        serde_json::to_string_pretty(&self.payload).unwrap_or_else(|_| "{}".to_string())
    }
}
