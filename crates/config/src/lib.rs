pub mod manager;
pub mod display_name;

pub use manager::ConfigManager;
pub use display_name::DisplayNameManager;

use terminal_lock_core::{error::*, paths::Paths, types::Preferences};

/// Load preferences from file
pub fn load_preferences(paths: &Paths) -> Result<Preferences> {
    let file = paths.preferences_file();
    
    if !file.exists() {
        return Ok(Preferences::default());
    }

    let content = std::fs::read_to_string(&file)?;
    let prefs = serde_json::from_str(&content)?;
    Ok(prefs)
}

/// Save preferences to file
pub fn save_preferences(paths: &Paths, prefs: &Preferences) -> Result<()> {
    let content = serde_json::to_string_pretty(prefs)?;
    std::fs::write(paths.preferences_file(), content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            paths.preferences_file(),
            std::fs::Permissions::from_mode(0o600),
        )?;
    }

    Ok(())
}