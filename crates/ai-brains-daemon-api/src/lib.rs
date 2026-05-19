use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::ingest::{IngestRequest, IngestResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonRequest {
    Ingest(IngestRequest),
    Sync(BridgeRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonResponse {
    Ingest(IngestResponse),
    Sync { success: bool },
}
