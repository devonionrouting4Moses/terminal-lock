/// Core types and traits for terminal-lock system
// Re-export core functionality from other crates
pub use terminal_lock_auth as auth;
pub use terminal_lock_core as core;
pub use terminal_lock_ui as ui;

// Re-export commonly used types from core
pub use terminal_lock_core::{
    error::{Error, Result},
    session::{Session, SessionManager},
};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Global configuration constants
pub mod constants {
    use std::time::Duration;

    /// Session timeout duration (15 minutes)
    pub const SESSION_TTL: Duration = Duration::from_secs(15 * 60);
    
    /// Maximum attempts per terminal session
    pub const MAX_ATTEMPTS_PER_SESSION: u32 = 3;
    
    /// Global maximum attempts before laptop lock
    pub const MAX_TOTAL_ATTEMPTS: u32 = 9;
    
    /// Lockout duration after failed attempts (60 seconds)
    pub const LOCKOUT_DURATION: Duration = Duration::from_secs(60);
    
    /// Password minimum length
    pub const PASSWORD_MIN_LENGTH: usize = 6;
    
    /// Password maximum length
    pub const PASSWORD_MAX_LENGTH: usize = 128;
    
    /// Salt length for password hashing
    pub const SALT_LENGTH: usize = 16;
}