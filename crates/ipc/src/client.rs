use crate::messages::{DaemonRequest, DaemonResponse};
use crate::socket_path;

#[cfg(unix)]
use crate::unix::UnixClient;

#[cfg(windows)]
use crate::windows::WindowsClient;

/// Daemon client with platform-specific implementations
pub struct DaemonClient;

impl DaemonClient {
    /// Connect to daemon and send a message with automatic platform selection
    /// 
    /// # Platform Behavior
    /// - Unix: Uses Unix domain sockets for efficient IPC
    /// - Windows: Uses TCP fallback (port 9999)
    /// 
    /// # Retry Logic
    /// - Attempts 3 connections with exponential backoff (100ms, 200ms)
    /// - Timeout: 5 seconds per request
    /// - Automatic fallback on failure
    pub fn send_message(msg: DaemonRequest) -> Result<DaemonResponse, Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            let socket_path = socket_path();
            UnixClient::send_request(&socket_path, msg)
        }
        
        #[cfg(windows)]
        {
            WindowsClient::send_request(msg)
        }
    }
}
