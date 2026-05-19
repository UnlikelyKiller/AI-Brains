use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_events::{Payload, SessionSummaryCreatedPayload};
use ai_brains_models::{CompletionRequest, ModelProvider, TokenizeRequest};
use ai_brains_store::{EventStore, QueryStore};
use std::str::FromStr;
use std::sync::Arc;

mod backup;
mod conflict_detection;
mod feedback_loop;
mod memory_synthesis;
mod recipe_promotion;
mod retention;

pub use backup::BackupService;
use conflict_detection::ConflictDetectionService;
pub use feedback_loop::FeedbackLoopService;
use memory_synthesis::MemorySynthesizer;
use recipe_promotion::RecipePromotionService;
pub use retention::RetentionService;

pub struct AggregatedLearningsService {
    query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
    model_provider: Arc<dyn ModelProvider>,
}

impl AggregatedLearningsService {
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

    pub async fn run_cross_agent_synthesis(
        &self,
        project_id: ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        tracing::info!("Starting Phase 15: Cross-Agent Memory Synthesis (Level 2)");
        let synthesizer = MemorySynthesizer::new(
            self.query_store.clone(),
            self.event_store.clone(),
            self.model_provider.clone(),
        );

        // Synthesize Level 1 -> Level 2
        synthesizer.run_synthesis(2, project_id).await
    }
}

pub struct NightlyService {
    query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
    completion_provider: Arc<dyn ModelProvider>,
    _embedding_provider: Arc<dyn ModelProvider>,
}

impl NightlyService {
    pub fn new(
        query_store: Arc<dyn QueryStore>,
        event_store: Arc<dyn EventStore>,
        completion_provider: Arc<dyn ModelProvider>,
        embedding_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            query_store,
            event_store,
            completion_provider,
            _embedding_provider: embedding_provider,
        }
    }

    pub async fn run_nightly(
        &self,
        project_id: ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let unsummarized = self.query_store.get_unsummarized_sessions()?;
        let mut count = 0;

        for session_id in unsummarized {
            if let Err(e) = self.summarize_session(&session_id).await {
                tracing::error!("Failed to summarize session {}: {}", session_id, e);
                continue;
            }
            count += 1;
        }

        // Run hierarchical synthesis
        let synthesizer = MemorySynthesizer::new(
            self.query_store.clone(),
            self.event_store.clone(),
            self.completion_provider.clone(),
        );
        if let Err(e) = synthesizer.run_synthesis(1, project_id).await {
            tracing::error!("Memory synthesis failed: {}", e);
        }

        // Retention Cleanup (90 days)
        let retention = RetentionService::new(self.query_store.clone(), 90);
        if let Err(e) = retention.run_cleanup().await {
            tracing::error!("Retention cleanup failed: {}", e);
        }

        // Cross-Agent Synthesis (Phase 15)
        let cross_agent = AggregatedLearningsService::new(
            self.query_store.clone(),
            self.event_store.clone(),
            self.completion_provider.clone(),
        );
        if let Err(e) = cross_agent.run_cross_agent_synthesis(project_id).await {
            tracing::error!("Cross-agent synthesis failed: {}", e);
        }

        // Feedback Loop Accuracy Check
        let feedback = FeedbackLoopService::new(self.query_store.clone(), self.event_store.clone());
        if let Err(e) = feedback.run_accuracy_check(project_id).await {
            tracing::error!("Feedback loop check failed: {}", e);
        }

        Ok(count)
    }

    async fn summarize_session(
        &self,
        session_id_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let turns = self.query_store.get_session_turns(session_id_str)?;
        if turns.is_empty() {
            return Ok(());
        }

        // Context Limit Configuration
        let ctx_limit = std::env::var("AI_BRAINS_CTX_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(38912);

        let overhead_buffer = 1500; // Buffer for system instructions, prompt headers, and response
        let effective_budget = ctx_limit.saturating_sub(overhead_buffer);
        tracing::info!(
            "Summarizing session {}: ctx_limit={}, effective_budget={}",
            session_id_str,
            ctx_limit,
            effective_budget
        );

        // Calculate current tokens
        let mut full_conversation = String::new();
        for (role, content) in &turns {
            full_conversation.push_str(&format!("{}: {}\n", role.to_uppercase(), content));
        }

        let token_count = self
            .completion_provider
            .tokenize(TokenizeRequest {
                text: full_conversation.clone(),
            })
            .await
            .map(|r| r.tokens.len())
            .unwrap_or_else(|_| ai_brains_models::estimate_tokens(&full_conversation));

        tracing::info!("Session {} token_count={}", session_id_str, token_count);

        if token_count > effective_budget {
            tracing::info!(
                "Session {} exceeds budget ({} > {}). Using sequential chunking.",
                session_id_str,
                token_count,
                effective_budget
            );
            return self
                .summarize_chunked(session_id_str, turns, effective_budget)
                .await;
        }

        let prompt = self.build_summary_prompt(&full_conversation, None);
        let response = self.execute_completion(prompt).await?;
        self.persist_and_follow_up(session_id_str, response.text)
            .await
    }

    async fn summarize_chunked(
        &self,
        session_id_str: &str,
        turns: Vec<(String, String)>,
        budget: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = self.split_into_chunks(turns, budget).await?;
        let mut previous_summary: Option<String> = None;
        let mut final_json: Option<String> = None;

        for (i, chunk_turns) in chunks.iter().enumerate() {
            tracing::info!(
                "Processing chunk {}/{} for session {}...",
                i + 1,
                chunks.len(),
                session_id_str
            );
            let mut chunk_text = String::new();
            for (role, content) in chunk_turns {
                chunk_text.push_str(&format!("{}: {}\n", role.to_uppercase(), content));
            }

            let prompt_header = format!(
                "This is Part {} of {} of a large session.\n\n\
                 {}\n\n\
                 Conversation Chunk:\n",
                i + 1,
                chunks.len(),
                previous_summary
                    .as_ref()
                    .map(|s| format!("Context from previous parts: {}\n", s))
                    .unwrap_or_default(),
            );

            let summary_prompt = self.build_summary_prompt(&chunk_text, Some(&prompt_header));
            let response = self.execute_completion(summary_prompt).await?;

            previous_summary = Some(response.text.clone());
            if i == chunks.len() - 1 {
                final_json = Some(response.text);
            }
        }

        if let Some(json) = final_json {
            self.persist_and_follow_up(session_id_str, json).await
        } else {
            Err("Failed to produce a final summary during chunking".into())
        }
    }

    fn build_summary_prompt(&self, conversation: &str, header_override: Option<&str>) -> String {
        let header = header_override.unwrap_or("Analyze the following developer session and extract a structured knowledge node in JSON format.\n\n");
        format!(
            "{}Rules:\n\
             1. Stick STRICTLY to facts in the provided text.\n\
             2. Do NOT hallucinate paths.\n\
             3. If a decision is not explicitly stated, do not include it.\n\n\
             JSON Schema:\n\
             {{\n\
               \"title\": \"Brief descriptive title\",\n\
               \"summary\": \"Concise overview of accomplishments\",\n\
               \"decisions\": [\"List of specific architectural or process decisions\"],\n\
               \"constraints\": [\"Technical constraints or invariants identified\"],\n\
               \"next_steps\": [\"Planned future work identified in session\"]\n\
             }}\n\n\
             Conversation:\n{}",
            header, conversation
        )
    }

    async fn split_into_chunks(
        &self,
        turns: Vec<(String, String)>,
        budget: usize,
    ) -> Result<Vec<Vec<(String, String)>>, Box<dyn std::error::Error>> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0;

        for turn in turns {
            let turn_text = format!("{}: {}\n", turn.0.to_uppercase(), turn.1);
            let turn_tokens = self
                .completion_provider
                .tokenize(TokenizeRequest {
                    text: turn_text.clone(),
                })
                .await
                .map(|r| r.tokens.len())
                .unwrap_or_else(|_| ai_brains_models::estimate_tokens(&turn_text));

            if current_tokens + turn_tokens > budget && !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = Vec::new();
                current_tokens = 0;
            }

            current_tokens += turn_tokens;
            current_chunk.push(turn);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        Ok(chunks)
    }

    async fn execute_completion(
        &self,
        prompt: String,
    ) -> Result<ai_brains_models::CompletionResponse, Box<dyn std::error::Error>> {
        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a factual data extraction engine for a technical memory vault. \
                 You output ONLY valid JSON. You are extremely conservative and avoid any unsupported claims.".to_string(),
            ),
            max_tokens: Some(1000),
            temperature: Some(0.0),
        };

        Ok(self.completion_provider.complete(request).await?)
    }

    async fn persist_and_follow_up(
        &self,
        session_id_str: &str,
        summary_json: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let memory_id = MemoryId::new();
        let session_id = SessionId::from_str(session_id_str)?;

        let event = ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionSummaryCreated,
            ai_brains_events::Actor::System,
            ai_brains_core::privacy::Privacy::LocalOnly,
        )
        .build(Payload::SessionSummaryCreated(
            SessionSummaryCreatedPayload {
                session_id,
                memory_id,
                summary: summary_json.clone(),
            },
        ))?;

        self.event_store.append_event(&event)?;

        // Run intelligence services
        let conflict_service = ConflictDetectionService::new(
            self.query_store.clone(),
            self.event_store.clone(),
            self.completion_provider.clone(),
        );
        let recipe_service = RecipePromotionService::new(
            self.query_store.clone(),
            self.event_store.clone(),
            self.completion_provider.clone(),
        );

        if let Err(e) = conflict_service
            .check_for_conflicts(&session_id, &summary_json)
            .await
        {
            tracing::error!(
                "Conflict detection failed for session {}: {}",
                session_id,
                e
            );
        }
        if let Err(e) = recipe_service
            .promote_recipes(&session_id, &summary_json)
            .await
        {
            tracing::error!("Recipe promotion failed for session {}: {}", session_id, e);
        }

        Ok(())
    }
}
