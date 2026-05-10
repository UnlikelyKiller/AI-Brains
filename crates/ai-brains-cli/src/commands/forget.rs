use crate::context::AppContext;
use ai_brains_core::ids::MemoryId;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, EventKind, MemoryForgottenPayload, Payload};
use ai_brains_store::EventStore;
use std::str::FromStr;

pub fn run(ctx: &AppContext, memory_id_str: String) -> Result<(), Box<dyn std::error::Error>> {
    let memory_id = MemoryId::from_str(&memory_id_str)?;
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let event = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryForgotten,
        Actor::User(ai_brains_core::ids::UserId::new()),
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryForgotten(MemoryForgottenPayload {
        memory_id,
    }))?;

    event_store.append_event(&event)?;

    println!("Memory {} marked as forgotten.", memory_id);
    Ok(())
}
