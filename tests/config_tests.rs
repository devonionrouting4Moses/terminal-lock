use terminal_lock_config::{load_preferences, save_preferences, ConfigManager, DisplayNameManager};
use terminal_lock_core::{
    types::{ColorScheme, GreetingStyle, Preferences, QuoteSource, AnimationType},
    Paths,
};

#[test]
fn test_default_preferences() {
    let prefs = Preferences::default();

    assert_eq!(prefs.color_scheme, ColorScheme::Vibrant);
    assert!(prefs.animations);
    assert_eq!(prefs.quote_source, QuoteSource::All);
    assert_eq!(prefs.greeting_style, GreetingStyle::Dynamic);
}

#[test]
fn test_preferences_serialization() {
    let prefs = Preferences::default();
    let json = serde_json::to_string(&prefs).unwrap();
    let deserialized: Preferences = serde_json::from_str(&json).unwrap();

    assert_eq!(prefs.color_scheme, deserialized.color_scheme);
    assert_eq!(prefs.animations, deserialized.animations);
    assert_eq!(prefs.quote_source, deserialized.quote_source);
    assert_eq!(prefs.greeting_style, deserialized.greeting_style);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_temp_paths() -> (TempDir, Paths) {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join(".terminal_lock");
        std::fs::create_dir_all(&lock_dir).unwrap();
        let paths = Paths::new().unwrap();
        (temp_dir, paths)
    }

    #[test]
    fn test_save_and_load_preferences() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        let prefs = Preferences {
            color_scheme: ColorScheme::Pastel,
            animations: false,
            animation_type: AnimationType::Starfield,
            quote_source: QuoteSource::Coding,
            greeting_style: GreetingStyle::Formal,
        };

        save_preferences(&paths, &prefs).unwrap();
        let loaded = load_preferences(&paths).unwrap();

        assert_eq!(prefs.color_scheme, loaded.color_scheme);
        assert_eq!(prefs.animations, loaded.animations);
        assert_eq!(prefs.animation_type, loaded.animation_type);
        assert_eq!(prefs.quote_source, loaded.quote_source);
        assert_eq!(prefs.greeting_style, loaded.greeting_style);
    }

    #[test]
    fn test_config_manager() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        let mut manager = ConfigManager::new(paths.clone()).unwrap();

        // Test color scheme change
        manager.set_color_scheme(ColorScheme::Minimal).unwrap();
        assert_eq!(manager.preferences().color_scheme, ColorScheme::Minimal);

        // Test animations toggle
        manager.set_animations(false).unwrap();
        assert!(!manager.preferences().animations);

        // Test quote source change
        manager.set_quote_source(QuoteSource::Spiritual).unwrap();
        assert_eq!(manager.preferences().quote_source, QuoteSource::Spiritual);

        // Test greeting style change
        manager.set_greeting_style(GreetingStyle::Casual).unwrap();
        assert_eq!(manager.preferences().greeting_style, GreetingStyle::Casual);

        // Test reset to defaults
        manager.reset_to_defaults().unwrap();
        assert_eq!(manager.preferences().color_scheme, ColorScheme::Vibrant);
        assert!(manager.preferences().animations);
    }

    #[test]
    fn test_display_name() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        let name = "TestUser";
        DisplayNameManager::save(&paths, name).unwrap();

        let loaded = DisplayNameManager::load(&paths).unwrap();
        assert_eq!(loaded, name);

        // Test empty name
        assert!(DisplayNameManager::save(&paths, "").is_err());
    }
}