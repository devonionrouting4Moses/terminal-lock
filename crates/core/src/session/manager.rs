use crate::error::Result;
use crate::paths::Paths;
use crate::types::SessionInfo;
use chrono::Utc;

/// Session manager for cleanup and maintenance
pub struct SessionManager;

impl SessionManager {
    /// Clean up old sessions
    pub fn cleanup_old_sessions(paths: &Paths) -> Result<()> {
        let now = Utc::now();
        
        for session_file in paths.all_session_files()? {
            if let Ok(content) = std::fs::read_to_string(&session_file) {
                if let Ok(session) = serde_json::from_str::<SessionInfo>(&content) {
                    if now > session.expiry {
                        let _ = std::fs::remove_file(&session_file);
                    }
                }
            }
        }

        Ok(())
    }

    /// Remove all sessions (for reset)
    pub fn remove_all_sessions(paths: &Paths) -> Result<()> {
        for session_file in paths.all_session_files()? {
            let _ = std::fs::remove_file(session_file);
        }
        Ok(())
    }
}
