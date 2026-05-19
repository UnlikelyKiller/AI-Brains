use ai_brains_core::ids::ProjectId;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{FeedbackMetricPayload, Payload};
use ai_brains_events::{Actor, AggregateType, EventKind};
use ai_brains_store::{EventStore, QueryStore};
use std::sync::Arc;

pub struct FeedbackLoopService {
    query_store: Arc<dyn QueryStore>,
    event_store: Arc<dyn EventStore>,
}

impl FeedbackLoopService {
    pub fn new(query_store: Arc<dyn QueryStore>, event_store: Arc<dyn EventStore>) -> Self {
        Self {
            query_store,
            event_store,
        }
    }

    pub async fn run_accuracy_check(
        &self,
        project_id: ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        tracing::info!(
            "Running Feedback Loop Accuracy Check for project: {}",
            project_id
        );

        // 1. Get recent ChangeGuard hotspots
        let hotspots = self.query_store.search_memories("HOTSPOT", 50)?;
        let hotspot_paths: Vec<String> =
            hotspots.iter().flat_map(|m| extract_paths(&m.1)).collect();

        if hotspot_paths.is_empty() {
            return Ok(0);
        }

        // 2. Get recent predictions (next_steps from summaries)
        // For simplicity, we search for memories containing "next steps" or "plan"
        let predictions = self.query_store.search_memories("next steps", 50)?;

        let mut matches = 0;
        for pred in predictions {
            let pred_paths = extract_paths(&pred.1);
            for path in pred_paths {
                if hotspot_paths.contains(&path) {
                    matches += 1;

                    // Record accuracy metric
                    let event = EventBuilder::new(
                        AggregateType::System,
                        uuid::Uuid::new_v4(),
                        EventKind::FeedbackMetric,
                        Actor::System,
                        ai_brains_core::privacy::Privacy::LocalOnly,
                    )
                    .build(Payload::FeedbackMetric(FeedbackMetricPayload {
                        metric_kind: "prediction_accuracy".to_string(),
                        value: format!("Matched predicted path: {}", path),
                        session_id: None,
                        project_id: Some(project_id),
                    }))?;

                    self.event_store.append_event(&event)?;
                }
            }
        }

        tracing::info!("Feedback loop found {} accuracy matches", matches);
        Ok(matches)
    }
}

fn extract_paths(content: &str) -> Vec<String> {
    // Simple heuristic for paths: words containing slashes or dots
    content
        .split_whitespace()
        .filter(|w| w.contains('/') || w.contains('\\') || w.contains('.'))
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '\\' && c != '.')
                .to_string()
        })
        .collect()
}
