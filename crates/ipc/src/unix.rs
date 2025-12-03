/// Unix domain socket implementation for IPC
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;
use crate::messages::{DaemonRequest, DaemonResponse};

/// Unix socket client for daemon communication
pub struct UnixClient;

impl UnixClient {
    /// Connect to daemon via Unix domain socket with automatic retry
    pub fn connect(socket_path: &PathBuf) -> Result<UnixStream, Box<dyn std::error::Error>> {
        let mut last_error = None;
        
        for attempt in 0..3 {
            match UnixStream::connect(socket_path) {
                Ok(stream) => {
                    // Set timeouts for reliability
                    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
                    return Ok(stream);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < 2 {
                        // Exponential backoff: 100ms, 200ms
                        std::thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                    }
                }
            }
        }
        
        Err(format!(
            "Failed to connect to daemon at {:?} after 3 attempts: {}",
            socket_path,
            last_error.map(|e| e.to_string()).unwrap_or_default()
        ).into())
    }

    /// Send request and receive response with length-prefixed protocol
    pub fn send_request(
        socket_path: &PathBuf,
        request: DaemonRequest,
    ) -> Result<DaemonResponse, Box<dyn std::error::Error>> {
        let mut stream = Self::connect(socket_path)?;
        
        // Serialize request to JSON
        let request_json = serde_json::to_string(&request)?;
        let request_bytes = request_json.as_bytes();
        let request_len = request_bytes.len() as u32;
        
        // Send length prefix (4 bytes, little-endian)
        stream.write_all(&request_len.to_le_bytes())?;
        stream.write_all(request_bytes)?;
        stream.flush()?;
        
        // Read response length
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)?;
        let response_len = u32::from_le_bytes(len_bytes) as usize;
        
        // Validate response size (max 1MB)
        if response_len > 1024 * 1024 {
            return Err("Response too large (>1MB)".into());
        }
        
        // Read response body
        let mut response_bytes = vec![0u8; response_len];
        stream.read_exact(&mut response_bytes)?;
        
        // Deserialize response
        let response = serde_json::from_slice(&response_bytes)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_construction() {
        let path = PathBuf::from("/tmp/test.sock");
        assert!(path.to_string_lossy().contains("test.sock"));
    }
}
