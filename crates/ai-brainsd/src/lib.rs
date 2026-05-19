use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink};
use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::ingest::{IngestRequest, IngestResponse};
use ai_brains_daemon_api::DaemonRequest;
use ai_brains_events::Envelope;
use ai_brains_store::event_store::EventStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

struct DaemonStoreSink {
    store: Arc<dyn EventStore>,
    last_error: Option<String>,
}

impl CaptureSink for DaemonStoreSink {
    fn append(&mut self, envelope: Envelope) {
        if let Err(e) = self.store.append_event(&envelope) {
            self.last_error = Some(e.to_string());
        }
    }
}

enum WriterMessage {
    Ingest {
        request: IngestRequest,
        spool_path: PathBuf,
        reply: oneshot::Sender<Result<IngestResponse, BoxError>>,
    },
    Sync {
        record: BridgeRecord,
        spool_path: PathBuf,
        reply: oneshot::Sender<Result<(), BoxError>>,
    },
}

#[derive(Clone)]
pub struct DaemonWriter {
    sender: mpsc::Sender<WriterMessage>,
    event_store: Arc<dyn EventStore>,
    spool_dir: PathBuf,
}

impl DaemonWriter {
    pub async fn start(
        spool_dir: PathBuf,
        event_store: Arc<dyn EventStore>,
    ) -> Result<Self, BoxError> {
        fs::create_dir_all(&spool_dir).await?;

        let (sender, mut receiver) = mpsc::channel(64);
        let worker_store = Arc::clone(&event_store);
        let worker_spool_dir = spool_dir.clone();

        tokio::spawn(async move {
            let service = CaptureService::new();
            replay_spool(&worker_spool_dir, &worker_store, &service)
                .await
                .ok();

            while let Some(message) = receiver.recv().await {
                match message {
                    WriterMessage::Ingest {
                        request,
                        spool_path,
                        reply,
                    } => {
                        let result =
                            process_ingest(&service, &worker_store, request, Some(spool_path))
                                .await;
                        let _ = reply.send(result);
                    }
                    WriterMessage::Sync {
                        record,
                        spool_path,
                        reply,
                    } => {
                        let result =
                            process_sync(&service, &worker_store, record, Some(spool_path)).await;
                        let _ = reply.send(result);
                    }
                }
            }
        });

        Ok(Self {
            sender,
            event_store,
            spool_dir,
        })
    }

    pub async fn ingest(&self, request: IngestRequest) -> Result<IngestResponse, BoxError> {
        let spool_path = self.spool_dir.join(format!("{}.json", Uuid::new_v4()));
        let payload = serde_json::to_vec(&DaemonRequest::Ingest(request.clone()))?;
        fs::write(&spool_path, payload).await?;

        let (reply_tx, reply_rx) = oneshot::channel();
        self.sender
            .send(WriterMessage::Ingest {
                request,
                spool_path,
                reply: reply_tx,
            })
            .await
            .map_err(|_| "daemon queue closed")?;

        reply_rx.await.map_err(|_| -> BoxError {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "daemon worker dropped",
            ))
        })?
    }

    pub async fn sync(&self, record: BridgeRecord) -> Result<(), BoxError> {
        let spool_path = self.spool_dir.join(format!("{}.json", Uuid::new_v4()));
        let payload = serde_json::to_vec(&DaemonRequest::Sync(record.clone()))?;
        fs::write(&spool_path, payload).await?;

        let (reply_tx, reply_rx) = oneshot::channel();
        self.sender
            .send(WriterMessage::Sync {
                record,
                spool_path,
                reply: reply_tx,
            })
            .await
            .map_err(|_| "daemon queue closed")?;

        reply_rx.await.map_err(|_| -> BoxError {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "daemon worker dropped",
            ))
        })?
    }

    pub async fn recorded_events(&self) -> Vec<Envelope> {
        self.event_store.read_all_events().unwrap_or_default()
    }

    pub fn spool_dir(&self) -> &Path {
        &self.spool_dir
    }
}

async fn replay_spool(
    spool_dir: &Path,
    store: &Arc<dyn EventStore>,
    service: &CaptureService,
) -> Result<(), BoxError> {
    let mut entries = fs::read_dir(spool_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path).await?;
        let request: DaemonRequest = serde_json::from_str(&content)?;
        match request {
            DaemonRequest::Ingest(ingest) => {
                process_ingest(service, store, ingest, Some(path)).await?;
            }
            DaemonRequest::Sync(record) => {
                process_sync(service, store, record, Some(path)).await?;
            }
        }
    }
    Ok(())
}

async fn process_ingest(
    service: &CaptureService,
    store: &Arc<dyn EventStore>,
    request: IngestRequest,
    spool_path: Option<PathBuf>,
) -> Result<IngestResponse, BoxError> {
    let mut sink = DaemonStoreSink {
        store: Arc::clone(store),
        last_error: None,
    };
    let outcome = service.ingest_request(request, CaptureContext::default(), &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist ingested turn: {}", err).into());
    }

    if let Some(path) = spool_path {
        let _ = fs::remove_file(path).await;
    }

    let event_id = outcome
        .events
        .first()
        .map(|e| e.event_id.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    Ok(IngestResponse {
        event_id,
        processed: true,
    })
}

async fn process_sync(
    service: &CaptureService,
    store: &Arc<dyn EventStore>,
    record: BridgeRecord,
    spool_path: Option<PathBuf>,
) -> Result<(), BoxError> {
    if record.direction != ai_brains_contracts::bridge::BridgeDirection::Inbound {
        if let Some(path) = spool_path {
            let _ = fs::remove_file(path).await;
        }
        return Ok(());
    }

    let mut sink = DaemonStoreSink {
        store: Arc::clone(store),
        last_error: None,
    };

    // Map record to IngestRequest (simplified for now, mirroring CLI logic)
    let role = match record.record_kind.to_lowercase().as_str() {
        "user" | "prompt" => "user",
        _ => "assistant",
    };

    let content = if let Some(s) = record.payload.as_str() {
        s.to_string()
    } else {
        record.payload.to_string()
    };

    let request = IngestRequest {
        session_id: record.session_id,
        project_id: record.project_id,
        harness_id: ai_brains_core::ids::HarnessId::default(),
        turn_id: ai_brains_core::ids::TurnId::new(),
        role: role.to_string(),
        content,
        thinking: None,
        privacy: record.privacy,
        tx_id: record.tx_id,
    };

    service.ingest_request(request, CaptureContext::default(), &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist synced record: {}", err).into());
    }

    if let Some(path) = spool_path {
        let _ = fs::remove_file(path).await;
    }

    Ok(())
}
