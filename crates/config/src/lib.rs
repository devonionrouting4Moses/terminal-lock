pub mod manager;
pub mod display_name;
pub mod preferences;

pub use manager::ConfigManager;
pub use display_name::DisplayNameManager;
pub use preferences::{save_preferences_to_db, get_preferences_from_db, parse_color_scheme, parse_greeting_style, parse_quote_source};

use terminal_lock_core::{error::*, paths::Paths, types::Preferences};

/// Load preferences from database via IPC
pub fn load_preferences(paths: &Paths) -> Result<Preferences> {
    // Return default preferences - actual loading should be done via IPC
    Ok(Preferences::default())
}

/// Save preferences to database via IPC
pub fn save_preferences(paths: &Paths, prefs: &Preferences) -> Result<()> {
    // Preferences should be saved via IPC to daemon database
    // This is a no-op now
    Ok(())
}