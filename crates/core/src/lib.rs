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

/// Global configuration constants
pub mod constants {
    use std::time::Duration;

    /// Session timeout duration (45 minutes)
    pub const SESSION_TTL: Duration = Duration::from_secs(45 * 60);
    
    /// Inactivity timeout duration (15 minutes)
    pub const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(15 * 60);
    
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