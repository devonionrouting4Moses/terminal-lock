pub mod messages;
pub mod client;
pub mod server;

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

pub use messages::{
    DaemonRequest, DaemonResponse, InvalidReason, SessionState, SessionStatus,
};
pub use client::DaemonClient;
pub use server::DaemonServer;

use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Get the socket path for the daemon
pub fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("terminal-lock.sock")
        } else {
            dirs::home_dir()
                .map(|h| h.join(".terminal_lock").join("daemon.sock"))
                .unwrap_or_else(|| PathBuf::from("/tmp/terminal-lock.sock"))
        }
    }
    
    #[cfg(windows)]
    {
        PathBuf::from("\\\\.\\pipe\\terminal_lock")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub terminal_pid: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: SessionStatus,
    pub lock_generation: u64,
}
