use terminal_lock_core::{error::*, paths::Paths};

/// Display name manager
pub struct DisplayNameManager;

impl DisplayNameManager {
    /// Load display name
    pub fn load(paths: &Paths) -> Result<String> {
        let file = paths.name_file();
        
        if !file.exists() {
            return Ok("User".to_string());
        }

        let name = std::fs::read_to_string(&file)?
            .trim()
            .to_string();

        Ok(if name.is_empty() { "User".to_string() } else { name })
    }

    /// Save display name
    pub fn save(paths: &Paths, name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(Error::InvalidPreference("Display name cannot be empty".into()));
        }

        std::fs::write(paths.name_file(), name.trim())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                paths.name_file(),
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        Ok(())
    }
}