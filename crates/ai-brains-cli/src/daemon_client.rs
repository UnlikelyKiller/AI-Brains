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

    pub fn spawn_daemon(
        &self,
        vault_path: &std::path::Path,
        key: &ai_brains_crypto::SqlCipherKey,
    ) -> std::io::Result<()> {
        let exe_path = std::env::current_exe()?;
        let daemon_name = if cfg!(windows) {
            "ai-brainsd.exe"
        } else {
            "ai-brainsd"
        };
        let mut daemon_path = exe_path
            .parent()
            .ok_or_else(|| std::io::Error::other("Failed to get executable parent dir"))?
            .to_path_buf();
        daemon_path.push(daemon_name);

        // Try next to current exe first
        let mut cmd = if daemon_path.exists() {
            std::process::Command::new(daemon_path)
        } else {
            // Fallback to searching PATH
            std::process::Command::new(daemon_name)
        };

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const DETACHED_PROCESS: u32 = 0x00000008;
            cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
        }

        cmd.env("AI_BRAINS_VAULT_PATH", vault_path)
            .env("AI_BRAINS_KEY", key.expose_secret())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        Ok(())
    }

    pub async fn ensure_running(
        &self,
        vault_path: &std::path::Path,
        key: &ai_brains_crypto::SqlCipherKey,
    ) -> bool {
        // First probe with ultra-fast timeout
        if self.probe(Duration::from_millis(10)).await {
            return true;
        }

        // Potential race: another process might be spawning the daemon right now.
        // Add a small jittered backoff and re-probe before attempting to spawn.
        let jitter = (std::process::id() % 50) as u64;
        tokio::time::sleep(Duration::from_millis(10 + jitter)).await;
        if self.probe(Duration::from_millis(10)).await {
            return true;
        }

        // Still not running, try to spawn
        if self.spawn_daemon(vault_path, key).is_ok() {
            // Give it some time to start and probe again
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_millis(50)).await;
                if self.probe(Duration::from_millis(10)).await {
                    return true;
                }
            }
        }

        false
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
