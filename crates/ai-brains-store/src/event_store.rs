use crate::connection::VaultConnection;
use crate::errors::{Result, StoreError};
use crate::projections;
use crate::SyncStateStore;
use ai_brains_events::Envelope;
use rusqlite::params;
use uuid::Uuid;

pub trait EventStore: Send + Sync {
    fn append_event(&self, envelope: &Envelope) -> Result<()>;
    fn read_events(&self, aggregate_id: Uuid) -> Result<Vec<Envelope>>;
    fn read_all_events(&self) -> Result<Vec<Envelope>>;
    fn get_sync_state(&self, key: &str) -> Result<Option<String>>;
    fn set_sync_state(&self, key: &str, value: &str) -> Result<()>;
    fn get_session_privacy(
        &self,
        session_id: &str,
    ) -> Result<Option<ai_brains_core::privacy::Privacy>>;
}

pub struct SqliteEventStore {
    pub conn: VaultConnection,
}

impl SqliteEventStore {
    pub fn new(conn: VaultConnection) -> Self {
        Self { conn }
    }

    pub fn connection(&self) -> &VaultConnection {
        &self.conn
    }
}

impl SyncStateStore for SqliteEventStore {
    fn set_sync_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock()?;
        conn.execute(
            "INSERT INTO sync_state (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

impl EventStore for SqliteEventStore {
    fn append_event(&self, envelope: &Envelope) -> Result<()> {
        let actor_json = serde_json::to_string(&envelope.actor)
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
        let payload_json = serde_json::to_string(&envelope.payload)
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
        let occurred_at = envelope
            .occurred_at
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|e| StoreError::EventAppendFailed(format!("Failed to format date: {}", e)))?;

        let aggregate_type_str = serde_json::to_string(&envelope.aggregate_type)
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?
            .trim_matches('"')
            .to_string();

        let event_type_str = serde_json::to_string(&envelope.event_type)
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?
            .trim_matches('"')
            .to_string();

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        tx.execute(
            "INSERT INTO events (
                event_id, schema_version, aggregate_type, aggregate_id, event_type,
                occurred_at, actor_json, causation_id, correlation_id, privacy,
                payload_json, payload_hash
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                envelope.event_id.to_string(),
                envelope.schema_version,
                aggregate_type_str,
                envelope.aggregate_id.to_string(),
                event_type_str,
                occurred_at,
                actor_json,
                envelope.causation_id.map(|u| u.to_string()),
                envelope.correlation_id.map(|u| u.to_string()),
                serde_json::to_string(&envelope.privacy)
                    .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?,
                payload_json,
                envelope.payload_hash,
            ],
        )
        .map_err(|e| {
            if e.to_string().contains("events are immutable") {
                StoreError::ImmutableEventModified(e.to_string())
            } else {
                StoreError::EventAppendFailed(e.to_string())
            }
        })?;

        // Apply projections
        projections::apply_all(&tx, envelope)?;

        tx.commit()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        Ok(())
    }

    fn read_events(&self, aggregate_id: Uuid) -> Result<Vec<Envelope>> {
        self.read_events_internal(Some(aggregate_id))
    }

    fn read_all_events(&self) -> Result<Vec<Envelope>> {
        self.read_events_internal(None)
    }

    fn get_sync_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock()?;
        let mut stmt = conn.prepare("SELECT value FROM sync_state WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    fn set_sync_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock()?;
        conn.execute(
            "INSERT INTO sync_state (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn get_session_privacy(
        &self,
        session_id: &str,
    ) -> Result<Option<ai_brains_core::privacy::Privacy>> {
        let conn = self.conn.lock()?;
        let mut stmt =
            conn.prepare("SELECT privacy FROM session_projection WHERE session_id = ?")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let p_str: String = row.get(0)?;
            let p: ai_brains_core::privacy::Privacy = serde_json::from_str(&p_str)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }
}

impl SqliteEventStore {
    fn read_events_internal(&self, aggregate_id: Option<Uuid>) -> Result<Vec<Envelope>> {
        let (query, params) = match aggregate_id {
            Some(id) => (
                "SELECT 
                    event_id, schema_version, aggregate_type, aggregate_id, event_type,
                    occurred_at, actor_json, causation_id, correlation_id, privacy,
                    payload_json, payload_hash
                FROM events 
                WHERE aggregate_id = ?
                ORDER BY occurred_at ASC",
                vec![id.to_string()],
            ),
            None => (
                "SELECT 
                    event_id, schema_version, aggregate_type, aggregate_id, event_type,
                    occurred_at, actor_json, causation_id, correlation_id, privacy,
                    payload_json, payload_hash
                FROM events 
                ORDER BY occurred_at ASC",
                vec![],
            ),
        };

        let conn = self.conn.lock()?;
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params))?;
        let mut events = Vec::new();

        while let Some(row) = rows.next()? {
            let event_id_str: String = row.get(0)?;
            let event_id = Uuid::parse_str(&event_id_str)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let schema_version: u32 = row.get(1)?;

            let aggregate_type_str: String = row.get(2)?;
            let aggregate_type = serde_json::from_str(&format!("\"{}\"", aggregate_type_str))
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let aggregate_id_str: String = row.get(3)?;
            let aggregate_id = Uuid::parse_str(&aggregate_id_str)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let event_type_str: String = row.get(4)?;
            let event_type = serde_json::from_str(&format!("\"{}\"", event_type_str))
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let occurred_at_str: String = row.get(5)?;
            let occurred_at = time::OffsetDateTime::parse(
                &occurred_at_str,
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let actor_json: String = row.get(6)?;
            let actor = serde_json::from_str(&actor_json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let causation_id_str: Option<String> = row.get(7)?;
            let causation_id = causation_id_str
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let correlation_id_str: Option<String> = row.get(8)?;
            let correlation_id = correlation_id_str
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let privacy_json: String = row.get(9)?;
            let privacy = serde_json::from_str(&privacy_json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let payload_json: String = row.get(10)?;
            let payload = serde_json::from_str(&payload_json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let payload_hash: String = row.get(11)?;

            events.push(Envelope {
                event_id,
                schema_version,
                aggregate_type,
                aggregate_id,
                event_type,
                occurred_at,
                actor,
                causation_id,
                correlation_id,
                privacy,
                payload,
                payload_hash,
            });
        }

        Ok(events)
    }
}
