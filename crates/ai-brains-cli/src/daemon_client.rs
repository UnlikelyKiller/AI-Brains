use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct DaemonClient {
    #[cfg(windows)]
    pipe_path: String,
    #[cfg(not(windows))]
    socket_path: String,
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            pipe_path: r"\\.\pipe\aibrains-sync".to_string(),
            #[cfg(not(windows))]
            socket_path: "/tmp/aibrains-sync.sock".to_string(),
        }
    }

    pub async fn probe(&self, timeout: Duration) -> bool {
        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ClientOptions;
            use tokio::time::timeout as tokio_timeout;

            let start = std::time::Instant::now();
            while start.elapsed() < timeout {
                match ClientOptions::new().open(&self.pipe_path) {
                    Ok(mut stream) => {
                        let ping = DaemonRequest::Ping;
                        if let Ok(json) = serde_json::to_vec(&ping) {
                            let mut payload = json;
                            payload.push(b'\n');

                            // Use tokio_timeout for write/read to stay within limits
                            let remaining = timeout.saturating_sub(start.elapsed());
                            if tokio_timeout(remaining, stream.write_all(&payload))
                                .await
                                .is_ok()
                            {
                                let mut buffer = [0u8; 1024];
                                let remaining = timeout.saturating_sub(start.elapsed());
                                if let Ok(Ok(n)) =
                                    tokio_timeout(remaining, stream.read(&mut buffer)).await
                                {
                                    if n > 0 {
                                        if let Ok(resp) =
                                            serde_json::from_slice::<DaemonResponse>(&buffer[..n])
                                        {
                                            if matches!(resp, DaemonResponse::Pong) {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return false;
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
            }
            false
        }

        #[cfg(not(windows))]
        {
            use tokio::net::UnixStream;
            use tokio::time::timeout as tokio_timeout;

            let start = std::time::Instant::now();
            while start.elapsed() < timeout {
                let remaining = timeout.saturating_sub(start.elapsed());
                if let Ok(Ok(mut stream)) =
                    tokio_timeout(remaining, UnixStream::connect(&self.socket_path)).await
                {
                    let ping = DaemonRequest::Ping;
                    if let Ok(json) = serde_json::to_vec(&ping) {
                        let mut payload = json;
                        payload.push(b'\n');

                        let remaining = timeout.saturating_sub(start.elapsed());
                        if tokio_timeout(remaining, stream.write_all(&payload))
                            .await
                            .is_ok()
                        {
                            let mut buffer = [0u8; 1024];
                            let remaining = timeout.saturating_sub(start.elapsed());
                            if let Ok(Ok(n)) =
                                tokio_timeout(remaining, stream.read(&mut buffer)).await
                            {
                                if n > 0 {
                                    if let Ok(resp) =
                                        serde_json::from_slice::<DaemonResponse>(&buffer[..n])
                                    {
                                        if matches!(resp, DaemonResponse::Pong) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return false;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            false
        }
    }
}
