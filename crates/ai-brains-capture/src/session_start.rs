use crate::command_handler::SessionStartCommand;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{Payload, SessionStartedPayload};
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind};

pub fn build_session_started(command: &SessionStartCommand) -> crate::Result<Envelope> {
    EventBuilder::new(
        AggregateType::Session,
        command.session_id.as_uuid(),
        EventKind::SessionStarted,
        Actor::Harness(command.harness_id),
        command.privacy,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id: command.session_id,
        project_id: command.project_id,
        tx_id: command.tx_id.clone(),
    }))
    .map_err(Into::into)
}
