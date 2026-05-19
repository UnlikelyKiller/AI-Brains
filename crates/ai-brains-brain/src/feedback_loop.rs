use ai_brains_core::ids::ProjectId;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{FeedbackMetricPayload, Payload};
use ai_brains_events::{Actor, AggregateType, EventKind};
use ai_brains_store::{EventStore, QueryStore};
use std::sync::Arc;

pub struct FeedbackLoopService {
    event_store: Arc<dyn EventStore>,
}

impl FeedbackLoopService {
    pub fn new(_query_store: Arc<dyn QueryStore>, event_store: Arc<dyn EventStore>) -> Self {
        Self { event_store }
    }

    pub async fn run_accuracy_check(
        &self,
        project_id: ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        tracing::info!(
            "Running Structured Feedback Loop Accuracy Check for project: {}",
            project_id
        );

        let all_events = self.event_store.read_all_events()?;

        // 1. Get recent Predictions
        let predictions: Vec<_> = all_events
            .iter()
            .filter_map(|e| {
                if let Payload::PredictionRecorded(p) = &e.payload {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        // 2. Get recent Verify Outcomes
        let outcomes: Vec<_> = all_events
            .iter()
            .filter_map(|e| {
                if let Payload::VerifyOutcomeRecorded(p) = &e.payload {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();

        if predictions.is_empty() || outcomes.is_empty() {
            return Ok(0);
        }

        let mut matches = 0;
        for pred in predictions {
            // Match outcomes by tx_id
            let matching_outcomes = outcomes.iter().filter(|o| {
                if let Some(pred_tx) = &pred.tx_id {
                    o.tx_id == *pred_tx
                } else {
                    false
                }
            });

            for outcome in matching_outcomes {
                if outcome.status == "passed" {
                    // Check if predicted paths overlap with affected paths
                    for path in &pred.predicted_paths {
                        if outcome.affected_paths.contains(path) {
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
                                metric_kind: "prediction_accuracy_structured".to_string(),
                                value: format!(
                                    "Match: Pred {} in Affected {}",
                                    path, outcome.tx_id
                                ),
                                session_id: Some(pred.session_id),
                                project_id: Some(project_id),
                            }))?;

                            self.event_store.append_event(&event)?;
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Structured feedback loop found {} accuracy matches",
            matches
        );
        Ok(matches)
    }
}
