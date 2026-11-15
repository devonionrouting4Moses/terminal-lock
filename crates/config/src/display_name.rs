use terminal_lock_core::{error::*, paths::Paths};

/// Display name manager
/// Deprecated - use IPC helper to load/save from database instead
pub struct DisplayNameManager;

impl DisplayNameManager {
    /// Load display name from database via IPC
    /// Deprecated - use IPC helper instead
    pub fn load(paths: &Paths) -> Result<String> {
        // Return default - actual loading should be done via IPC
        Ok("User".to_string())
    }

    /// Save display name to database via IPC
    /// Deprecated - use IPC helper instead
    pub fn save(paths: &Paths, name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::InvalidPreference("Display name cannot be empty".into()));
        }

        // Display name should be saved via IPC to daemon database
        // This is a no-op now
        Ok(())
    }
}