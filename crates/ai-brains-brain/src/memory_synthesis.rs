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

    pub async fn run_synthesis(
        &self,
        target_level: u32,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if target_level == 0 {
            return Ok(0);
        }

        // 1. Get all memories at source level that haven't been synthesized yet
        let source_level = target_level - 1;
        let source_memories = self.query_store.get_memories_by_level(source_level)?;

        // Find memories that aren't already parents in the hierarchy
        // For now, we'll just synthesize what we have if we have enough.
        // A more advanced implementation would track 'new' memories specifically.
        if source_memories.len() < 2 {
            return Ok(0);
        }

        // 2. Cluster them
        let clusters = self.cluster_memories(&source_memories).await?;

        let mut count = 0;
        for cluster in clusters {
            if cluster.len() < 2 {
                continue;
            }

            // 3. Summarize the cluster
            let synthesis = self.synthesize_cluster(&cluster, target_level).await?;

            // 4. CRAG: Verify the synthesis
            if !self.verify_synthesis(&cluster, &synthesis).await? {
                tracing::warn!(
                    "Synthesized level {} memory was rejected by CRAG verification: {}",
                    target_level,
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
                level: target_level,
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
        level: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut contents = String::new();
        for (_, content) in cluster {
            contents.push_str("- ");
            contents.push_str(content);
            contents.push('\n');
        }

        let role = if level == 1 {
            "synthesizing developer session history"
        } else {
            "aggregating high-level architectural and process learnings"
        };

        let prompt = format!(
            "Synthesize the following related level {} memories into a single, higher-level knowledge node (Level {}). \
             Focus on recurring patterns, shared technical context, and cumulative progress across sessions and agents.\n\n\
             Memories:\n{}",
            level - 1, level, contents
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(format!(
                "You are a principal engineer {} into a knowledge base.",
                role
            )),
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
