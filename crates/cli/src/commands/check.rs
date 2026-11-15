use terminal_lock_core::{error::Result, Paths};
use crate::ipc_helper;

/// Check if current session is valid
/// Returns exit code 0 if session is valid
/// Returns exit code 1 if session is invalid/expired
/// This is used to block terminal usage when session is not valid
/// 
/// Checks:
/// - Session file exists (or daemon has session)
/// - Session hasn't expired (45-minute hard timeout)
/// - Session hasn't been idle for 15+ minutes
/// - Session lock generation matches current lock generation (not from before a lock)
pub fn run(paths: &Paths) -> Result<()> {
    // Get shell PID from environment variable
    let shell_pid = std::env::var("TERMINAL_LOCK_SESSION_PID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| std::process::id());
    
    eprintln!("DEBUG: Check command called for shell PID {}", shell_pid);
    
    // Check if session is valid via IPC with fallback to file-based
    match ipc_helper::validate_session_with_fallback(shell_pid, paths) {
        Ok(true) => {
            // Session is valid - allow command execution
            eprintln!("DEBUG: ✅ Session is valid");
            Ok(())
        }
        Ok(false) => {
            // Session is invalid - block command execution
            eprintln!("DEBUG: ❌ Session is invalid - terminal will exit");
            eprintln!("❌ Session expired or invalid. Please authenticate again.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("ERROR: Failed to check session: {}", e);
            Err(e)
        }
    }
}
