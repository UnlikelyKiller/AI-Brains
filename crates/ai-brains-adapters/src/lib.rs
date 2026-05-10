mod adapter;
mod antigravity;
mod capability;
mod claude;
mod codex;
mod config_patch;
mod errors;
mod gemini;
mod hook_output;
mod install;
mod neutral_event;
mod opencode;
mod wrapper;

pub use adapter::{adapter_capability, AdapterKind};
pub use antigravity::{
    antigravity_capability, discover_sessions, extract_turns, filter_recent_sessions,
    import_antigravity_sessions, manual_import_instructions, parse_overview_file,
    session_id_from_path, AntigravityStep, AntigravityTurn,
};
pub use capability::{AdapterCapability, CapabilityLevel};
pub use claude::parse_claude_stop_payload;
pub use config_patch::apply_idempotent_patch;
pub use errors::{AdapterError, Result};
pub use hook_output::render_hook_output;
pub use install::install_scope;
pub use neutral_event::NeutralEvent;
pub use wrapper::wrapper_command;
