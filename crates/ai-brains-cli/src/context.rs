use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, EventKind, Payload, ProjectRegisteredPayload};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::EventStore;
use rusqlite::params;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppContext {
    pub vault_path: PathBuf,
    pub _key: SqlCipherKey,
    pub conn: Arc<VaultConnection>,
}

impl AppContext {
    pub fn from_cli(
        vault_path: Option<PathBuf>,
        key: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path =
            vault_path.ok_or("Vault path is required (--vault-path or AI_BRAINS_VAULT_PATH)")?;

        // In degraded mode, we use a fixed dummy key if none provided
        // This allows rusqlite-bundled to work even if SQLCipher isn't active
        let key_str = key.unwrap_or_else(|| {
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
        });

        let key = SqlCipherKey::from_raw(key_str);
        let conn = VaultConnection::open(path.clone(), &key)?;
        conn.migrate()?;

        Ok(Self {
            vault_path: path,
            _key: key,
            conn: Arc::new(conn),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ensure_project_and_session_exists(
        &self,
        sink: &mut StoreSink,
        service: &ai_brains_capture::CaptureService,
        capture_context: &ai_brains_capture::CaptureContext,
        project_id: ai_brains_core::ids::ProjectId,
        session_id: ai_brains_core::ids::SessionId,
        harness_id: ai_brains_core::ids::HarnessId,
        privacy: ai_brains_core::privacy::Privacy,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Auto-create project if it doesn't exist
        let project_exists = {
            let conn_lock = self.conn.lock()?;
            let mut stmt =
                conn_lock.prepare("SELECT 1 FROM project_projection WHERE project_id = ?")?;
            stmt.exists(params![project_id.to_string()])?
        };

        if !project_exists {
            let event = EventBuilder::new(
                AggregateType::Project,
                project_id.as_uuid(),
                EventKind::ProjectRegistered,
                Actor::User(ai_brains_core::ids::UserId::new()),
                privacy,
            )
            .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
                project_id,
                name: format!("Project {}", project_id),
            }))?;

            sink.store.append_event(&event)?;
        }

        // Auto-start session if it doesn't exist
        let session_exists = {
            let conn_lock = self.conn.lock()?;
            let mut stmt =
                conn_lock.prepare("SELECT 1 FROM session_projection WHERE session_id = ?")?;
            stmt.exists(params![session_id.to_string()])?
        };

        if !session_exists {
            service.start_session(
                ai_brains_capture::SessionStartCommand {
                    session_id,
                    project_id,
                    harness_id,
                    privacy,
                },
                capture_context.clone(),
                sink,
            )?;
        }

        Ok(())
    }
}

pub struct StoreSink {
    pub store: ai_brains_store::SqliteEventStore,
    pub last_error: Option<String>,
}

impl ai_brains_capture::CaptureSink for StoreSink {
    fn append(&mut self, envelope: ai_brains_events::Envelope) {
        if let Err(err) = self.store.append_event(&envelope) {
            self.last_error = Some(err.to_string());
        }
    }
}
