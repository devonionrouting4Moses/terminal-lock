use terminal_lock_core::{error::Result, Paths};
use crate::ipc_helper;

/// Update session activity timestamp (does NOT extend expiry)
/// This is called after each command execution to update the last activity time
/// Sessions have a FIXED 45-minute expiry that never extends
pub fn run(paths: &Paths) -> Result<()> {
    // Get shell PID from environment variable
    let shell_pid = std::env::var("TERMINAL_LOCK_SESSION_PID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| std::process::id());
    
    // Validate session and update activity timestamp via IPC with fallback
    // The 45-minute expiry time is NEVER extended - it's a hard limit
    // Only the last_activity timestamp is updated for inactivity tracking
    // If session is invalid/expired, do nothing (user will need to re-authenticate)
    let _ = ipc_helper::extend_session_with_fallback(shell_pid, paths);
    
    Ok(())
}
