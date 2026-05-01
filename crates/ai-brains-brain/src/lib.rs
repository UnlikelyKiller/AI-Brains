use ai_brains_core::ids::{MemoryId, SessionId};
use ai_brains_events::{Payload, SessionSummaryCreatedPayload};
use ai_brains_models::{CompletionRequest, ModelProvider};
use ai_brains_store::{EventStore, QueryStore};
use std::str::FromStr;
use std::sync::Arc;

mod backup;
mod conflict_detection;
mod memory_synthesis;
mod recipe_promotion;
mod retention;

pub use backup::BackupService;
use conflict_detection::ConflictDetectionService;
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

    pub async fn run_cross_agent_synthesis(&self) -> Result<usize, Box<dyn std::error::Error>> {
        tracing::info!("Starting Phase 15: Cross-Agent Memory Synthesis (Level 2)");
        let synthesizer = MemorySynthesizer::new(
            self.query_store.clone(),
            self.event_store.clone(),
            self.model_provider.clone(),
        );

        // Synthesize Level 1 -> Level 2
        synthesizer.run_synthesis(2).await
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

    pub async fn run_nightly(&self) -> Result<usize, Box<dyn std::error::Error>> {
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
        if let Err(e) = synthesizer.run_synthesis(1).await {
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
        if let Err(e) = cross_agent.run_cross_agent_synthesis().await {
            tracing::error!("Cross-agent synthesis failed: {}", e);
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

        let mut conversation = String::new();
        for (role, content) in turns {
            conversation.push_str(&format!("{}: {}\n", role.to_uppercase(), content));
        }

        let prompt = format!(
            "Analyze the following developer session and extract a structured knowledge node in JSON format.\n\n\
             Rules:\n\
             1. Stick STRICTLY to facts in the provided text.\n\
             2. Do NOT hallucinate paths (e.g. do not assume ~/.config if the source says C:\\Users).\n\
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
            conversation
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a factual data extraction engine for a technical memory vault. \
                 You output ONLY valid JSON. You are extremely conservative and avoid any unsupported claims.".to_string(),
            ),
            max_tokens: Some(600),
            temperature: Some(0.0), // Maximum deterministic for facts
        };

        let response = self.completion_provider.complete(request).await?;
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
                summary: response.text.clone(),
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

        let summary_text = response.text;
        if let Err(e) = conflict_service
            .check_for_conflicts(&session_id, &summary_text)
            .await
        {
            tracing::error!(
                "Conflict detection failed for session {}: {}",
                session_id,
                e
            );
        }
        if let Err(e) = recipe_service
            .promote_recipes(&session_id, &summary_text)
            .await
        {
            tracing::error!("Recipe promotion failed for session {}: {}", session_id, e);
        }

        Ok(())
    }
}
