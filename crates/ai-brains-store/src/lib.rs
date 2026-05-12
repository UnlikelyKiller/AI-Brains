pub mod config;
pub mod connection;
pub mod errors;
pub mod event_store;
pub mod fts;
pub mod migrations;
pub mod pragmas;
pub mod projections;
pub mod query_store;
pub mod replay;
pub mod transaction;

pub use connection::VaultConnection;
pub use errors::{Result, StoreError};
pub use event_store::{EventStore, SqliteEventStore};
pub use fts::{FtsSearch, SearchResult};
pub use transaction::Transaction;

use ai_brains_core::ids::{MemoryId, SessionId};

pub trait QueryStore: std::marker::Send + std::marker::Sync {
    fn get_unsummarized_sessions(&self) -> Result<Vec<String>>;
    fn get_session_turns(&self, session_id: &str) -> Result<Vec<(String, String)>>;
    fn get_session_status(&self, session_id: &SessionId) -> Result<Option<String>>;
    fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<(MemoryId, String)>>;
    fn get_memories_by_level(&self, level: u32) -> Result<Vec<(MemoryId, String)>>;
    fn delete_old_turns(&self, cutoff: chrono::DateTime<chrono::Utc>) -> Result<usize>;
    fn update_memory_status(&self, memory_id: &MemoryId, status: &str) -> Result<()>;
    fn list_forgotten_memories(
        &self,
        project_id: Option<ai_brains_core::ids::ProjectId>,
    ) -> Result<Vec<(String, String)>>;
}
