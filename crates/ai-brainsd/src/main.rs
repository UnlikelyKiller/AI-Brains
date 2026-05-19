use ai_brains_crypto::SqlCipherKey;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brainsd::DaemonWriter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    let writer = DaemonWriter::start(spool_dir, event_store).await?;
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

async fn handle_client(
    mut server: tokio::net::windows::named_pipe::NamedPipeServer,
    writer: DaemonWriter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 4096];

    // Simple robust read loop: read until connection is closed or we have a valid JSON
    // Note: Named pipes on Windows can be tricky with EOF.
    // For this bridge, we assume one request per connection for simplicity, or
    // we would need a proper framing protocol.

    loop {
        let n = server.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);

        // Attempt to parse. If it fails, keep reading until EOF or valid.
        if let Ok(request) = serde_json::from_slice::<DaemonRequest>(&buffer) {
            match request {
                DaemonRequest::Ingest(req) => {
                    let resp = writer.ingest(req).await?;
                    let payload = serde_json::to_vec(&DaemonResponse::Ingest(resp))?;
                    server.write_all(&payload).await?;
                }
                DaemonRequest::Sync(record) => {
                    writer.sync(record).await?;
                    let payload = serde_json::to_vec(&DaemonResponse::Sync { success: true })?;
                    server.write_all(&payload).await?;
                }
            }
            break; // Finished handling the request
        }
    }

    server.flush().await?;
    Ok(())
}
