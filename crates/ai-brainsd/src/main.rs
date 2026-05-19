use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brainsd::DaemonWriter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(windows)]
use tokio::net::windows::named_pipe::ServerOptions;

#[tokio::main]
#[allow(clippy::disallowed_methods)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let mut spool_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    spool_dir.push(".ai-brains");
    spool_dir.push("spool");

    let vault_path = std::env::var("AI_BRAINS_VAULT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".ai-brains");
            path.push("vault.db");
            path
        });

    let vault_key_str = std::env::var("AI_BRAINS_VAULT_KEY").unwrap_or_else(|_| {
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
    });

    let key = SqlCipherKey::from_raw(vault_key_str);
    let conn = VaultConnection::open(vault_path, &key)?;
    conn.migrate()?;

    let event_store = Arc::new(SqliteEventStore::new(conn));
    let writer = DaemonWriter::start(spool_dir, event_store.clone()).await?;

    #[cfg(windows)]
    {
        let pipe_name = r"\\.\pipe\aibrains-sync";
        println!("AI-Brains Daemon started. Listening on {}", pipe_name);

        loop {
            let server = match ServerOptions::new()
                .first_pipe_instance(false)
                .create(pipe_name)
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create named pipe instance: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            if let Err(e) = server.connect().await {
                eprintln!("Failed to connect client: {}", e);
                continue;
            }

            let writer_clone = writer.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_client(server, writer_clone).await {
                    eprintln!("Error handling client: {}", e);
                }
            });
        }
    }

    #[cfg(not(windows))]
    {
        let socket_path = "/tmp/aibrains-sync.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = tokio::net::UnixListener::bind(socket_path)?;
        println!(
            "AI-Brains Daemon started. Listening on Unix socket: {}",
            socket_path
        );

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let writer_clone = writer.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, writer_clone).await {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept UDS connection: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
}

#[allow(clippy::disallowed_methods)]
async fn handle_client<S>(
    mut server: S,
    writer: DaemonWriter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 4096];

    loop {
        let n = server.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);

        if buffer.len() > 8 * 1024 * 1024 {
            return Err("Buffer exceeded 8 MiB limit. Disconnecting.".into());
        }

        // Process newline-delimited JSON records
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_with_nl = buffer.drain(..pos + 1).collect::<Vec<u8>>();
            let line = &line_with_nl[..line_with_nl.len() - 1];
            if line.is_empty() {
                continue;
            }

            let request = match serde_json::from_slice::<DaemonRequest>(line) {
                Ok(request) => Some(request),
                Err(_) => {
                    // Try parsing as raw BridgeRecord directly
                    match serde_json::from_slice::<ai_brains_contracts::bridge::BridgeRecord>(line)
                    {
                        Ok(record) => Some(DaemonRequest::Sync(record)),
                        Err(e) => {
                            eprintln!(
                                "Failed to parse as either DaemonRequest or BridgeRecord: {}",
                                e
                            );
                            None
                        }
                    }
                }
            };

            if let Some(request) = request {
                match request {
                    DaemonRequest::Ingest(req) => {
                        let resp = writer.ingest(req).await?;
                        let mut payload = serde_json::to_vec(&DaemonResponse::Ingest(resp))?;
                        payload.push(b'\n');
                        server.write_all(&payload).await?;
                    }
                    DaemonRequest::Sync(record) => {
                        if record.record_kind == "query" {
                            let query_text = record
                                .payload
                                .get("text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            // Parse string IDs from interchange format for internal use.
                            use std::str::FromStr;
                            let project_id = ai_brains_core::ids::ProjectId::from_str(&record.project_id)
                                .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new());
                            let session_id = match &record.session_id {
                                Some(s) => ai_brains_core::ids::SessionId::from_str(s)
                                    .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new()),
                                None => ai_brains_core::ids::SessionId::new(),
                            };

                            let hits = writer
                                .query_memories(query_text, project_id, session_id)
                                .await?;

                            let timestamp = chrono::Utc::now().to_rfc3339();

                            for h in hits {
                                let payload = serde_json::json!({
                                    "type": "Insight",
                                    "memory_id": h.memory_id,
                                    "relevance": h.score.unwrap_or(1.0),
                                    "content": h.content
                                });

                                let resp_record = BridgeRecord {
                                    bridge_version: "0.2".to_string(),
                                    direction: BridgeDirection::Outbound,
                                    timestamp: timestamp.clone(),
                                    parent_hash: None,
                                    project_id: record.project_id.clone(),
                                    session_id: record.session_id.clone(),
                                    tx_id: None,
                                    record_kind: "insight".to_string(),
                                    payload,
                                    privacy: ai_brains_core::privacy::Privacy::LocalOnly,
                                };

                                let mut payload = serde_json::to_vec(&resp_record)?;
                                payload.push(b'\n');
                                server.write_all(&payload).await?;
                            }
                            server.write_all(b"\n").await?;
                        } else {
                            writer.sync(record).await?;
                            let mut payload =
                                serde_json::to_vec(&DaemonResponse::Sync { success: true })?;
                            payload.push(b'\n');
                            server.write_all(&payload).await?;
                        }
                    }
                }
            }
        }
    }

    server.flush().await?;
    Ok(())
}
