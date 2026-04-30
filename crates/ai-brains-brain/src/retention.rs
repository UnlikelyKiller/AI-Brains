use ai_brains_store::QueryStore;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{error, info};

pub struct RetentionService {
    query_store: Arc<dyn QueryStore>,
    retention_days: i64,
}

impl RetentionService {
    pub fn new(query_store: Arc<dyn QueryStore>, retention_days: i64) -> Self {
        Self {
            query_store,
            retention_days,
        }
    }

    pub async fn run_cleanup(&self) -> Result<usize, Box<dyn std::error::Error>> {
        info!(
            "Starting raw turn retention cleanup ({} days)...",
            self.retention_days
        );

        let cutoff = Utc::now() - Duration::days(self.retention_days);
        match self.query_store.delete_old_turns(cutoff) {
            Ok(count) => {
                info!("Cleaned up {} expired turns.", count);
                Ok(count)
            }
            Err(e) => {
                error!("Retention cleanup failed: {}", e);
                Err(e.into())
            }
        }
    }
}
