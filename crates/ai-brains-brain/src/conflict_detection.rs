use ai_brains_core::ids::{ConflictId, SessionId};
use ai_brains_events::{ConflictDetectedPayload, Payload};
use ai_brains_models::{CompletionRequest, ModelProvider};
use ai_brains_store::{EventStore, QueryStore};
use std::sync::Arc;

pub struct ConflictDetectionService {
    query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
    model_provider: Arc<dyn ModelProvider>,
}

impl ConflictDetectionService {
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

    pub async fn check_for_conflicts(
        &self,
        session_id: &SessionId,
        summary: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Retrieve potentially related memories via FTS
        // Create a more permissive query by joining words with OR
        let clean_query = summary
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect::<String>();
        let fts_query = clean_query
            .split_whitespace()
            .take(10)
            .collect::<Vec<_>>()
            .join(" OR ");

        let related_memories = self.query_store.search_memories(&fts_query, 5)?;

        if related_memories.is_empty() {
            return Ok(());
        }

        let mut context = String::new();
        for (id, content) in &related_memories {
            context.push_str(&format!("Memory {}: {}\n", id, content));
        }

        let prompt = format!(
            "Analyze if the following new session summary contradicts any of the existing memories.\n\nNew Summary: {}\n\nExisting Memories:\n{}\n\nIf there is a direct contradiction, explain why. If not, respond with 'NO CONFLICT'.",
            summary,
            context
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a helpful assistant detecting contradictions in developer knowledge."
                    .to_string(),
            ),
            max_tokens: Some(300),
            temperature: Some(0.1),
        };

        let response = self.model_provider.complete(request).await?;

        if response.text.to_uppercase().contains("NO CONFLICT") {
            return Ok(());
        }

        // 2. Emit ConflictDetected event
        let conflict_id = ConflictId::new();
        let event = ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Conflict,
            conflict_id.as_uuid(),
            ai_brains_events::EventKind::ConflictDetected,
            ai_brains_events::Actor::System,
            ai_brains_core::privacy::Privacy::LocalOnly,
        )
        .build(Payload::ConflictDetected(ConflictDetectedPayload {
            conflict_id,
            session_id: *session_id,
            contradicted_memory_ids: related_memories.iter().map(|(id, _)| *id).collect(),
            explanation: response.text,
        }))?;

        self.event_store.append_event(&event)?;

        Ok(())
    }
}
