/// IPC Helper - Provides daemon communication with file-based fallback
use terminal_lock_core::{error::Result, paths::Paths, session::Session};
use terminal_lock_ipc::{DaemonClient, DaemonRequest, DaemonResponse};
use std::time::SystemTime;

/// Try to validate session via daemon, fallback to file-based
pub fn validate_session_with_fallback(shell_pid: u32, paths: &Paths) -> Result<bool> {
    // Try daemon first
    let request = DaemonRequest::ValidateSession { shell_pid };
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::SessionValid { .. }) => {
            eprintln!("✅ Session valid [daemon]");
            Ok(true)
        }
        Ok(DaemonResponse::SessionInvalid { .. }) => {
            eprintln!("❌ Session invalid [daemon]");
            Ok(false)
        }
        Ok(_) => {
            eprintln!("⚠️  Unexpected daemon response [fallback to file-based]");
            Session::is_valid_with_lock_check(paths)
        }
        Err(e) => {
            eprintln!("⚠️  Daemon unavailable: {} [fallback to file-based]", e);
            // Fallback to file-based session check
            Session::is_valid_with_lock_check(paths)
        }
    }
}

/// Try to extend session via daemon, fallback to file-based
pub fn extend_session_with_fallback(shell_pid: u32, paths: &Paths) -> Result<()> {
    // Try daemon first
    let request = DaemonRequest::ExtendSession { shell_pid };
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::SessionExtended { session_id }) => {
            eprintln!("✅ Session extended: {} [daemon]", session_id);
            Ok(())
        }
        Ok(_) => {
            eprintln!("⚠️  Unexpected daemon response [fallback to file-based]");
            let _ = Session::validate_and_extend(paths)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("⚠️  Daemon unavailable: {} [fallback to file-based]", e);
            // Fallback to file-based session extend
            let _ = Session::validate_and_extend(paths)?;
            Ok(())
        }
    }
}

/// Try to create session via daemon, fallback to file-based
pub fn create_session_with_fallback(shell_pid: u32, password_hash: &str, paths: &Paths) -> Result<String> {
    // Try daemon first
    let request = DaemonRequest::CreateSession {
        shell_pid,
        password_hash: password_hash.to_string(),
    };
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::SessionCreated { session_id, expires_at }) => {
            let secs_remaining = expires_at
                .duration_since(SystemTime::now())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            eprintln!("✅ Session created via daemon: {} (expires in {} seconds)", session_id, secs_remaining);
            Ok(session_id)
        }
        Ok(_) => {
            eprintln!("⚠️  Unexpected daemon response [fallback to file-based]");
            Session::create(paths)?;
            Ok("file-based".to_string())
        }
        Err(e) => {
            eprintln!("⚠️  Daemon unavailable: {} [fallback to file-based]", e);
            // Fallback to file-based session creation
            Session::create(paths)?;
            Ok("file-based".to_string())
        }
    }
}

/// Get session status from daemon
pub fn get_session_status(shell_pid: Option<u32>) -> Result<Vec<terminal_lock_ipc::SessionStatus>> {
    let request = DaemonRequest::GetSessionStatus { shell_pid };
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::SessionStatus { sessions }) => {
            Ok(sessions)
        }
        Ok(_) => {
            eprintln!("⚠️  Unexpected daemon response");
            Err(terminal_lock_core::error::Error::Unknown("Unexpected daemon response".to_string()))
        }
        Err(e) => {
            eprintln!("⚠️  Cannot get session status: {}", e);
            Err(terminal_lock_core::error::Error::Unknown(format!("IPC error: {}", e)))
        }
    }
}

/// Check if daemon is running
pub fn is_daemon_running() -> bool {
    let request = DaemonRequest::Ping;
    
    match DaemonClient::send_message(request) {
        Ok(DaemonResponse::Pong) => true,
        _ => false,
    }
}
