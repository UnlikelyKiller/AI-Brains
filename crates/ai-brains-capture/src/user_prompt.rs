use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{Payload, UserPromptRecordedPayload};
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind};

pub fn build_user_prompt(
    request: &IngestRequest,
    privacy: ai_brains_core::privacy::Privacy,
) -> crate::Result<Envelope> {
    EventBuilder::new(
        AggregateType::Session,
        request.session_id.as_uuid(),
        EventKind::UserPromptRecorded,
        Actor::Harness(request.harness_id),
        privacy,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: request.session_id,
        content: request.content.clone(),
        tx_id: request.tx_id.clone(),
    }))
    .map_err(Into::into)
}
