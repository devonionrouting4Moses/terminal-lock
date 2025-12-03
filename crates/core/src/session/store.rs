use crate::error::Result;
use crate::paths::Paths;

/// Session storage operations
pub struct SessionStore;

impl SessionStore {
    /// Clean up old session files (from previous shell instances)
    /// This prevents accumulation of session files when sourcing shell config
    /// CRITICAL: Only delete if it's NOT our session file
    pub fn cleanup_old_sessions(paths: &Paths, our_session_filename: &str) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(paths.lock_dir()) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.starts_with("session.") && filename.ends_with(".json") {
                                // CRITICAL: Only delete if it's NOT our session file
                                if filename != our_session_filename {
                                    eprintln!("DEBUG: Cleaning up old session file: {}", filename);
                                    let _ = std::fs::remove_file(entry.path());
                                } else {
                                    eprintln!("DEBUG: Skipping our session file: {}", filename);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
