use ai_brains_core::ids::{MemoryId, ProjectId};
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
        project_id: ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if target_level == 0 {
            return Ok(0);
        }

        // 1. Get all memories at source level that haven't been synthesized yet
        let source_level = target_level - 1;
        let source_memories = self.query_store.get_memories_by_level(source_level)?;

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
                project_id,
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
            contents.push_str("--- SOURCE MEMORY ---\n");
            contents.push_str(content);
            contents.push('\n');
        }

        let prompt = format!(
            "Synthesize the following Level {} memories into a single Level {} Knowledge Node in JSON format.\n\n\
             Rules:\n\
             1. Aggregate recurring patterns and permanent constraints.\n\
             2. Maintain technical density.\n\
             3. Output ONLY valid JSON.\n\n\
             JSON Schema:\n\
             {{\n\
               \"title\": \"Synthesis Title\",\n\
               \"aggregated_context\": \"Combined summary of work\",\n\
               \"invariants\": [\"Shared technical invariants identified across sessions\"],\n\
               \"cumulative_progress\": [\"Overall progress made across these nodes\"]\n\
             }}\n\n\
             Source Memories:\n{}",
            level - 1, level, contents
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a factual synthesis engine for a hierarchical knowledge vault. You output ONLY valid JSON.".to_string(),
            ),
            max_tokens: Some(1000),
            temperature: Some(0.0),
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
            sources.push_str("--- SOURCE ---\n");
            sources.push_str(content);
            sources.push('\n');
        }

        let prompt = format!(
            "Perform a rigorous factual audit of the following JSON synthesis against its sources.\n\n\
             Check for:\n\
             1. Factual contradictions.\n\
             2. Hallucinations (e.g. paths, features, or events NOT in the sources).\n\
             3. Over-reaching claims.\n\n\
             If the JSON is factually grounded in the sources, respond with 'SUPPORTED'.\n\
             If it contains any unsupported claims, respond with 'UNSUPPORTED' and list the errors.\n\n\
             Source Data:\n{}\n\nSynthesis JSON:\n{}",
            sources, synthesis
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a strict technical auditor. You verify facts and reject hallucinations."
                    .to_string(),
            ),
            max_tokens: Some(200),
            temperature: Some(0.0),
        };

        let response = self.model_provider.complete(request).await?;
        let text = response.text.to_uppercase();

        if text.contains("UNSUPPORTED") {
            tracing::warn!("CRAG REJECTED: {}", response.text);
            return Ok(false);
        }

        Ok(text.contains("SUPPORTED"))
    }
}
