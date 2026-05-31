use ai_brains_models::{EmbeddingRequest, ModelProvider};
use ai_brains_store::QueryStore;
use std::sync::Arc;

/// Service for generating and storing embeddings for vault memories.
pub struct EmbeddingService {
    query_store: Arc<dyn QueryStore>,
    embedding_provider: Arc<dyn ModelProvider>,
}

impl EmbeddingService {
    pub fn new(
        query_store: Arc<dyn QueryStore>,
        embedding_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            query_store,
            embedding_provider,
        }
    }

    /// Generate and store embedding for a single memory.
    /// Non-fatal: returns Ok(()) even if the LLM call fails.
    pub async fn generate_and_store(
        &self,
        memory_id: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = if content.len() > 4000 {
            &content[..4000]
        } else {
            content
        };

        let request = EmbeddingRequest {
            text: text.to_string(),
        };

        match self.embedding_provider.embed(request).await {
            Ok(response) => {
                let bytes = f32_vec_to_bytes(&response.vector);
                if let Err(e) = self.query_store.store_embedding(memory_id, &bytes) {
                    tracing::warn!("Failed to store embedding for memory {}: {}", memory_id, e);
                } else {
                    tracing::info!("Stored embedding for memory {}", memory_id);
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to generate embedding for memory {}: {}",
                    memory_id,
                    e
                );
            }
        }

        Ok(())
    }

    /// Backfill embeddings for recent memories without embeddings.
    pub async fn backfill_recent(
        &self,
        limit: usize,
        since_days: Option<i32>,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let memories = self
            .query_store
            .get_memories_without_embeddings(limit, since_days)?;

        if memories.is_empty() {
            return Ok((0, 0));
        }

        eprintln!(
            "[Nightly] Backfilling embeddings for {} memories...",
            memories.len()
        );

        let mut success = 0;
        let mut failed = 0;

        for (memory_id, content) in memories {
            match self.generate_and_store(&memory_id, &content).await {
                Ok(()) => success += 1,
                Err(_) => failed += 1,
            }

            // Brief yield to avoid overwhelming the embedding server
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        eprintln!(
            "[Nightly] Embedding backfill complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed))
    }

    /// Refresh stale embeddings (older than threshold)
    pub async fn refresh_stale(
        &self,
        days_threshold: i32,
        limit: usize,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let memories = self.query_store.get_stale_memories(days_threshold, limit)?;

        if memories.is_empty() {
            return Ok((0, 0));
        }

        eprintln!(
            "[Nightly] Refreshing {} stale embeddings (>{} days old)...",
            memories.len(),
            days_threshold
        );

        let mut success = 0;
        let mut failed = 0;

        for (idx, (memory_id, content)) in memories.iter().enumerate() {
            eprintln!(
                "    [Stale {}/{}] Re-embedding memory {}...",
                idx + 1,
                memories.len(),
                &memory_id[..8.min(memory_id.len())]
            );

            match self.generate_and_store(memory_id, content).await {
                Ok(()) => success += 1,
                Err(_) => failed += 1,
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        eprintln!(
            "[Nightly] Stale refresh complete: {} succeeded, {} failed.",
            success, failed
        );
        Ok((success, failed))
    }
}

fn f32_vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &v in vec {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}
