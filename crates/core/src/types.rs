use serde::{Deserialize, Serialize};

/// Color scheme for terminal display
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    Vibrant,
    Pastel,
    Minimal,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::Vibrant
    }
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vibrant => write!(f, "vibrant"),
            Self::Pastel => write!(f, "pastel"),
            Self::Minimal => write!(f, "minimal"),
        }
    }
}

/// Quote source category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuoteSource {
    All,
    Coding,
    Spiritual,
    Motivational,
    General,
}

impl Default for QuoteSource {
    fn default() -> Self {
        Self::All
    }
}

impl std::fmt::Display for QuoteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Coding => write!(f, "coding"),
            Self::Spiritual => write!(f, "spiritual"),
            Self::Motivational => write!(f, "motivational"),
            Self::General => write!(f, "general"),
        }
    }
}

/// Greeting style
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GreetingStyle {
    Dynamic,
    Formal,
    Casual,
}

impl Default for GreetingStyle {
    fn default() -> Self {
        Self::Dynamic
    }
}

impl std::fmt::Display for GreetingStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dynamic => write!(f, "dynamic"),
            Self::Formal => write!(f, "formal"),
            Self::Casual => write!(f, "casual"),
        }
    }
}

/// Animation types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AnimationType {
    Matrix,
    Starfield,
    Particles,
}

impl Default for AnimationType {
    fn default() -> Self {
        Self::Matrix
    }
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub color_scheme: ColorScheme,
    pub animations: bool,
    pub animation_type: AnimationType,
    pub quote_source: QuoteSource,
    pub greeting_style: GreetingStyle,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Vibrant,
            animations: true,
            animation_type: AnimationType::Matrix,
            quote_source: QuoteSource::All,
            greeting_style: GreetingStyle::Dynamic,
        }
    }
}

/// Quote with author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub text: String,
    pub author: Option<String>,
}

impl Quote {
    pub fn new(text: String, author: Option<String>) -> Self {
        Self { text, author }
    }

    pub fn fallback() -> Self {
        Self {
            text: "The best way to predict the future is to invent it.".to_string(),
            author: Some("Alan Kay".to_string()),
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub expiry: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub terminal_id: String,
    #[serde(default)]
    pub lock_generation: u64,
}

/// Lockout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutInfo {
    pub until: chrono::DateTime<chrono::Utc>,
    pub terminal_id: String,
}

/// Authentication attempts tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptsInfo {
    pub total: u32,
    pub last_boot_time: i64,
}

impl Default for AttemptsInfo {
    fn default() -> Self {
        Self {
            total: 0,
            last_boot_time: 0,
        }
    }
}