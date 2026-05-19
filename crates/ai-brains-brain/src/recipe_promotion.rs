use ai_brains_core::ids::{RecipeId, SessionId};
use ai_brains_events::{Payload, RecipePromotedPayload};
use ai_brains_models::{CompletionRequest, ModelProvider};
use ai_brains_store::{EventStore, QueryStore};
use std::sync::Arc;

pub struct RecipePromotionService {
    _query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
    model_provider: Arc<dyn ModelProvider>,
}

impl RecipePromotionService {
    pub fn new(
        query_store: Arc<dyn QueryStore>,
        event_store: Arc<dyn EventStore>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            _query_store: query_store,
            event_store,
            model_provider,
        }
    }

    pub async fn promote_recipes(
        &self,
        _session_id: &SessionId,
        summary: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Check if the session contains a "how-to" or a "workaround" that should be a recipe
        let prompt = format!(
            "Analyze the following session summary. If it contains a clear sequence of steps for a workaround, setup, or recurring task, extract them as a 'Recipe'.\n\nSummary: {}\n\nIf a recipe is found, provide a name and a bulleted list of steps. If not, respond with 'NO RECIPE'.",
            summary
        );

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(
                "You are a helpful assistant extracting execution recipes from developer sessions."
                    .to_string(),
            ),
            max_tokens: Some(400),
            temperature: Some(0.2),
        };

        let response = self.model_provider.complete(request).await?;

        if response.text.to_uppercase().contains("NO RECIPE") {
            return Ok(());
        }

        // 2. Parse the recipe (simple heuristic for now)
        let lines: Vec<&str> = response.text.lines().collect();
        if lines.is_empty() {
            return Ok(());
        }

        let name = lines[0].trim().replace("Name: ", "");
        let steps: Vec<String> = lines
            .iter()
            .skip(1)
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.starts_with('-')
                    || trimmed.starts_with('*')
                    || (!trimmed.is_empty()
                        && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()))
            })
            .map(|l| {
                l.trim()
                    .trim_start_matches(|c: char| !c.is_alphanumeric())
                    .trim()
                    .to_string()
            })
            .collect();

        if steps.is_empty() {
            return Ok(());
        }

        // 3. Emit RecipePromoted event
        let recipe_id = RecipeId::new();
        let event = ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Recipe,
            recipe_id.as_uuid(),
            ai_brains_events::EventKind::RecipePromoted,
            ai_brains_events::Actor::System,
            ai_brains_core::privacy::Privacy::LocalOnly,
        )
        .build(Payload::RecipePromoted(RecipePromotedPayload {
            recipe_id,
            name,
            content: steps.join("\n"),
            steps,
            source_memory_ids: Vec::new(), // Link to source memories if we had them
        }))?;

        self.event_store.append_event(&event)?;

        Ok(())
    }
}
