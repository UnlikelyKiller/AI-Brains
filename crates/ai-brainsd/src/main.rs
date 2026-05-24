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

type BoxError = Box<dyn std::error::Error + Send + Sync>;

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

    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);

    #[cfg(windows)]
    {
        let pipe_name = r"\\.\pipe\aibrains-sync";
        println!("AI-Brains Daemon started. Listening on {}", pipe_name);

        let writer_clone = writer.clone();
        let shutdown_tx_clone = shutdown_tx.clone();

        tokio::spawn(async move {
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

                tokio::select! {
                    res = server.connect() => {
                        if let Err(e) = res {
                            eprintln!("Failed to connect client: {}", e);
                            continue;
                        }

                        let writer_inner = writer_clone.clone();
                        let mut shutdown_rx_inner = shutdown_tx_clone.subscribe();
                        tokio::spawn(async move {
                            tokio::select! {
                                _ = handle_client(server, writer_inner) => {}
                                _ = shutdown_rx_inner.recv() => {
                                    tracing::info!("Shutting down client connection...");
                                }
                            }
                        });
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutdown signal received. Closing daemon...");
                        let _ = shutdown_tx_clone.send(());
                        break;
                    }
                }
            }
        });
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

        let writer_clone = writer.clone();
        let shutdown_tx_clone = shutdown_tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    res = listener.accept() => {
                        match res {
                            Ok((stream, _addr)) => {
                                let writer_inner = writer_clone.clone();
                                let mut shutdown_rx_inner = shutdown_tx_clone.subscribe();
                                tokio::spawn(async move {
                                    tokio::select! {
                                        _ = handle_client(stream, writer_inner) => {}
                                        _ = shutdown_rx_inner.recv() => {
                                            tracing::info!("Shutting down client connection...");
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to accept UDS connection: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutdown signal received. Closing daemon...");
                        let _ = shutdown_tx_clone.send(());
                        break;
                    }
                }
            }
        });
    }

    // Wait for shutdown signal in the main task too
    let _ = tokio::signal::ctrl_c().await;
    let _ = shutdown_tx.send(());

    #[cfg(not(windows))]
    {
        let socket_path = "/tmp/aibrains-sync.sock";
        let _ = std::fs::remove_file(socket_path);
    }

    // Give some time for background tasks to finish
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    println!("Daemon exited cleanly.");
    Ok(())
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
                let result: Result<(), BoxError> = match request {
                    DaemonRequest::Ping => {
                        let mut payload = serde_json::to_vec(&DaemonResponse::Pong)?;
                        payload.push(b'\n');
                        server.write_all(&payload).await?;
                        Ok(())
                    }
                    DaemonRequest::Ingest(req) => match writer.ingest(req).await {
                        Ok(resp) => {
                            let mut payload = serde_json::to_vec(&DaemonResponse::Ingest(resp))?;
                            payload.push(b'\n');
                            server.write_all(&payload).await?;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                    DaemonRequest::Sync(record) => {
                        if record.record_kind == "query" {
                            let payload = record.payload_value();
                            let query_text =
                                payload.get("text").and_then(|v| v.as_str()).unwrap_or("");

                            // Parse string IDs from interchange format for internal use.
                            use std::str::FromStr;
                            let project_id =
                                ai_brains_core::ids::ProjectId::from_str(&record.project_id)
                                    .unwrap_or_else(|_| ai_brains_core::ids::ProjectId::new());
                            let session_id = match &record.session_id {
                                Some(s) => ai_brains_core::ids::SessionId::from_str(s)
                                    .unwrap_or_else(|_| ai_brains_core::ids::SessionId::new()),
                                None => ai_brains_core::ids::SessionId::new(),
                            };

                            match writer
                                .query_memories(query_text, project_id, session_id)
                                .await
                            {
                                Ok(hits) => {
                                    let timestamp = chrono::Utc::now();

                                    for h in hits {
                                        let payload =
                                            ai_brains_contracts::bridge::BridgePayload::Insight {
                                                type_field: "Insight".to_string(),
                                                memory_id: h.memory_id,
                                                relevance: h.score.unwrap_or(1.0),
                                                content: h.content,
                                            };

                                        let resp_record = BridgeRecord {
                                            bridge_version: "0.3".to_string(),
                                            direction: BridgeDirection::Outbound,
                                            timestamp,
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
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        } else {
                            match writer.sync(record).await {
                                Ok(_) => {
                                    let mut payload = serde_json::to_vec(&DaemonResponse::Sync {
                                        success: true,
                                    })?;
                                    payload.push(b'\n');
                                    server.write_all(&payload).await?;
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        }
                    }
                };

                if let Err(e) = result {
                    let api_err =
                        ai_brains_contracts::response::ApiError::new("DAEMON_ERROR", e.to_string());
                    let resp = DaemonResponse::Error(api_err);
                    if let Ok(mut payload) = serde_json::to_vec(&resp) {
                        payload.push(b'\n');
                        let _ = server.write_all(&payload).await;
                    }
                }
            }
        }
    }

    server.flush().await?;
    Ok(())
}
