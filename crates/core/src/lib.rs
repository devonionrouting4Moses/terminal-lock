// Core library implementation
/// Core types and traits for terminal-lock system
pub mod error;
pub mod paths;
pub mod session;
pub mod types;
pub mod lock_monitor;
pub mod config;

pub use error::{Error, Result};
pub use paths::Paths;
pub use session::{Session, SessionManager};
pub use types::*;
pub use lock_monitor::LockMonitor;

// use std::time::Duration;

/// Global configuration constants - loaded from database
pub mod constants {
    use std::time::Duration;

    // Fallback defaults (only used if database is unavailable)
    pub const DEFAULT_SESSION_TTL_SECS: u64 = 45 * 60;
    pub const DEFAULT_INACTIVITY_TIMEOUT_SECS: u64 = 15 * 60;
    pub const DEFAULT_MAX_ATTEMPTS_PER_SESSION: u32 = 3;
    pub const DEFAULT_MAX_TOTAL_ATTEMPTS: u32 = 9;
    pub const DEFAULT_LOCKOUT_DURATION_SECS: u64 = 60;

    /// Session timeout duration (45 minutes) - loaded from database
    pub const SESSION_TTL: Duration = Duration::from_secs(DEFAULT_SESSION_TTL_SECS);
    
    /// Inactivity timeout duration (15 minutes) - loaded from database
    pub const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(DEFAULT_INACTIVITY_TIMEOUT_SECS);
    
    /// Maximum attempts per terminal session - loaded from database
    pub const MAX_ATTEMPTS_PER_SESSION: u32 = DEFAULT_MAX_ATTEMPTS_PER_SESSION;
    
    /// Global maximum attempts before laptop lock - loaded from database
    pub const MAX_TOTAL_ATTEMPTS: u32 = DEFAULT_MAX_TOTAL_ATTEMPTS;
    
    /// Lockout duration after failed attempts (60 seconds) - loaded from database
    pub const LOCKOUT_DURATION: Duration = Duration::from_secs(DEFAULT_LOCKOUT_DURATION_SECS);
    
    /// Password minimum length
    pub const PASSWORD_MIN_LENGTH: usize = 6;
    
    /// Password maximum length
    pub const PASSWORD_MAX_LENGTH: usize = 128;
    
    /// Salt length for password hashing
    pub const SALT_LENGTH: usize = 16;
}