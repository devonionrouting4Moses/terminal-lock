use terminal_lock_core::{
    error::*,
    paths::Paths,
    types::{ColorScheme, GreetingStyle, Preferences, QuoteSource, AnimationType},
};

/// Configuration manager
pub struct ConfigManager {
    paths: Paths,
    preferences: Preferences,
}

impl ConfigManager {
    pub fn new(paths: Paths) -> Result<Self> {
        let preferences = crate::load_preferences(&paths)?;
        Ok(Self { paths, preferences })
    }

    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) -> Result<()> {
        self.preferences.color_scheme = scheme;
        self.save()
    }

    pub fn set_animations(&mut self, enabled: bool) -> Result<()> {
        self.preferences.animations = enabled;
        self.save()
    }

    pub fn set_animation_type(&mut self, anim_type: AnimationType) -> Result<()> {
        self.preferences.animation_type = anim_type;
        self.save()
    }

    pub fn set_quote_source(&mut self, source: QuoteSource) -> Result<()> {
        self.preferences.quote_source = source;
        self.save()
    }

    pub fn set_greeting_style(&mut self, style: GreetingStyle) -> Result<()> {
        self.preferences.greeting_style = style;
        self.save()
    }

    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.preferences = Preferences::default();
        self.save()
    }

    pub fn export_to_file(&self, path: &std::path::Path) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.preferences)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn import_from_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.preferences = serde_json::from_str(&content)?;
        self.save()
    }

    fn save(&self) -> Result<()> {
        crate::save_preferences(&self.paths, &self.preferences)
    }
}