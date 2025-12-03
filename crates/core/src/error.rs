use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Terminal locked until {0}")]
    TerminalLocked(chrono::DateTime<chrono::Local>),

    #[error("System locked: too many failed attempts")]
    SystemLocked,

    #[error("Configuration not found. Run setup first.")]
    ConfigNotFound,

    #[error("Invalid password format: {0}")]
    InvalidPasswordFormat(String),

    #[error("Password mismatch")]
    PasswordMismatch,

    #[error("Invalid preference value: {0}")]
    InvalidPreference(String),

    #[error("Quote API error: {0}")]
    QuoteApiError(String),

    #[error("Animation error: {0}")]
    AnimationError(String),

    #[error("Not an interactive shell")]
    NotInteractive,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;