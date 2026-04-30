use ai_brains_core::ids::MemoryId;
use ai_brains_events::{MemorySynthesizedPayload, Payload};
use ai_brains_models::{CompletionRequest, ModelProvider};
use ai_brains_store::{EventStore, QueryStore};
use std::sync::Arc;

pub struct MemorySynthesizer {
    query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
    model_provider: Arc<dyn ModelProvider>,
}

impl MemorySynthesizer {
    pub fn new(
        query_store: Arc<dyn QueryStore>,
        event_store: Arc<dyn EventStore>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            query_store,
            event_store,
            model_provider,
        }
    }

    pub async fn run_synthesis(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // 1. Get all memories at level 0 that haven't been synthesized yet
        // For now, we'll just get all level 0 memories and cluster them.
        // In a real system, we'd track which ones are 'processed'.
        let level_0_memories = self.query_store.get_memories_by_level(0)?;
        if level_0_memories.len() < 2 {
            return Ok(0);
        }

        // 2. Cluster them
        // For level-1 clustering, we'll use a simple heuristic for now:
        // Group by 5 memories each or use the LLM to find groups.
        // Real RAPTOR uses GMM on embeddings.
        let clusters = self.cluster_memories(&level_0_memories).await?;

        let mut count = 0;
        for cluster in clusters {
            if cluster.len() < 2 {
                continue;
            }

            // 3. Summarize the cluster
            let synthesis = self.synthesize_cluster(&cluster).await?;

            // 4. CRAG: Verify the synthesis
            if !self.verify_synthesis(&cluster, &synthesis).await? {
                tracing::warn!(
                    "Synthesized memory was rejected by CRAG verification: {}",
                    synthesis
                );
                continue;
            }

            // 5. Emit event
            let memory_id = MemoryId::new();
            let source_memory_ids = cluster.iter().map(|(id, _)| *id).collect();

            let event = ai_brains_events::constructors::EventBuilder::new(
                ai_brains_events::AggregateType::Memory,
                memory_id.as_uuid(),
                ai_brains_events::EventKind::MemorySynthesized,
                ai_brains_events::Actor::System,
                ai_brains_core::privacy::Privacy::LocalOnly,
            )
            .build(Payload::MemorySynthesized(MemorySynthesizedPayload {
                memory_id,
                content: synthesis,
                source_memory_ids,
                level: 1,
            }))?;

            self.event_store.append_event(&event)?;
            count += 1;
        }

        Ok(count)
    }

    async fn cluster_memories(
        &self,
        memories: &[(MemoryId, String)],
    ) -> Result<Vec<Vec<(MemoryId, String)>>, Box<dyn std::error::Error>> {
        // Heuristic: Group by 5 for now.
        // TODO: Use embeddings and GMM/K-Means.
        let mut clusters = Vec::new();
        for chunk in memories.chunks(5) {
            clusters.push(chunk.to_vec());
        }
        Ok(clusters)
    }

    async fn synthesize_cluster(
        &self,
        cluster: &[(MemoryId, String)],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut contents = String::new();
        for (_, content) in cluster {
            contents.push_str("- ");
            contents.push_str(content);
            contents.push('\n');
        }

        let prompt = format!(
            "Synthesize the following related session summaries into a single, high-level knowledge node. \
             Focus on recurring patterns, shared technical context, and cumulative progress.\n\n\
             Summaries:\n{}",
            contents
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some("You are a principal engineer synthesizing developer session history into a knowledge base.".to_string()),
            max_tokens: Some(400),
            temperature: Some(0.3),
        };

        let response = self.model_provider.complete(request).await?;
        Ok(response.text)
    }

    async fn verify_synthesis(
        &self,
        cluster: &[(MemoryId, String)],
        synthesis: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut sources = String::new();
        for (_, content) in cluster {
            sources.push_str("- ");
            sources.push_str(content);
            sources.push('\n');
        }

        let prompt = format!(
            "Verify the following synthesized summary against the source summaries. \
             Does the synthesis accurately reflect the information provided? \
             If it introduces hallucinations or unsupported claims, respond with 'UNSUPPORTED'. \
             Otherwise, respond with 'SUPPORTED'.\n\n\
             Source Summaries:\n{}\n\nSynthesized Summary: {}",
            sources, synthesis
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some("You are a factual verification assistant.".to_string()),
            max_tokens: Some(50),
            temperature: Some(0.0),
        };

        let response = self.model_provider.complete(request).await?;
        let text = response.text.to_uppercase();
        Ok(text.contains("SUPPORTED") && !text.contains("UNSUPPORTED"))
    }
}
