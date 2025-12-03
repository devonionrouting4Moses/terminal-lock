use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Messages sent from CLI to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonRequest {
    /// Ping daemon to check if it's alive
    Ping,
    
    /// Create new session after successful authentication
    CreateSession {
        shell_pid: u32,
        password_hash: String,
    },
    
    /// Validate an existing session
    ValidateSession {
        shell_pid: u32,
    },
    
    /// Update session activity (extend idle timeout)
    ExtendSession {
        shell_pid: u32,
    },

    /// Invalidate a session (e.g., on lock)
    InvalidateSession {
        shell_pid: u32,
    },  
    
    /// Get session status
    GetSessionStatus {
        shell_pid: Option<u32>,
    },

    /// List all active sessions
    ListSessions,
    
    /// Pause all sessions (on system lock)
    PauseAllSessions,
    
    /// Resume sessions (on system unlock)
    ResumeAllSessions,

    /// Increment lock generation
    IncrementLockGeneration,

    /// Manually lock all sessions
    LockAllSessions,
    
    /// System lock detected (called by lock monitor)
    SystemLocked,
    
    /// System unlocked (called by lock monitor)
    SystemUnlocked,
    
    /// Shutdown daemon
    Shutdown,
}

/// Messages sent from daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    /// Pong response
    Pong,
    
    /// Session created successfully
    SessionCreated {
        session_id: String,
        expires_at: SystemTime,
    },
    
    /// Session is valid
    SessionValid {
        session_id: String,
        time_remaining_secs: u64,
        lock_generation: u64,
    },
    
    /// Session is invalid
    SessionInvalid {
        reason: InvalidReason,
    },
    
    /// Session updated
    SessionExtended {
        session_id: String,
    },
    
    /// Session invalidated
    SessionInvalidated {
        shell_pid: u32,
    },
    
    /// Session status information
    SessionStatus {
        sessions: Vec<SessionStatus>,
    },

    /// All sessions paused
    AllSessionsPaused,

    /// All sessions resumed
    AllSessionsResumed,
    
    /// Lock generation incremented
    LockGenerationIncremented {
        new_generation: u64,
    },
    
    /// All sessions locked
    AllSessionsLocked,
    
    /// System lock acknowledged
    SystemLockAcknowledged,
    
    /// System unlock acknowledged
    SystemUnlockAcknowledged,
    
    /// Daemon shutdown initiated
    ShutdownAcknowledged,
    
    /// Error occurred
    Error {
        message: String,
    },
}

/// Reasons why a session might be invalid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidReason {
    /// No session exists for this shell PID
    NotFound,
    
    /// Session expired (15-minute hard timeout)
    Expired,
    
    /// Session idle for too long (5-minute idle timeout)
    IdleTimeout,
    
    /// Session invalidated by system lock
    SystemLocked,
    
    /// Session manually locked
    ManuallyLocked,
    
    /// Lock generation mismatch
    LockGenerationMismatch {
        session_gen: u64,
        current_gen: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListEntry {
    pub shell_pid: u32,
    pub session_id: String,
    pub lock_generation: u64,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub last_activity: SystemTime,
    pub state: SessionState,
}

/// Session status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    /// Unique session ID
    pub id: String,
    
    /// Shell PID this session belongs to
    pub shell_pid: u32,
    
    /// When session was created
    pub created_at: SystemTime,
    
    /// When session expires (15 minutes from creation)
    pub expires_at: SystemTime,
    
    /// Last activity timestamp
    pub last_activity: SystemTime,
    
    /// Current session state
    pub state: SessionState,
    
    /// Lock generation when session was created
    pub lock_generation: u64,
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active and valid
    Active,
    
    /// Session is paused (system locked)
    Paused,
    
    /// Session has expired
    Expired,
    
    /// Session is locked (idle timeout or manual)
    Locked,
}

impl SessionState {
    pub fn is_valid(&self) -> bool {
        matches!(self, SessionState::Active)
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Active => write!(f, "Active"),
            SessionState::Paused => write!(f, "Paused"),
            SessionState::Expired => write!(f, "Expired"),
            SessionState::Locked => write!(f, "Locked"),
        }
    }
}

impl std::fmt::Display for InvalidReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidReason::NotFound => write!(f, "No session found"),
            InvalidReason::Expired => write!(f, "Session expired (15-minute timeout)"),
            InvalidReason::IdleTimeout => write!(f, "Session idle for 5+ minutes"),
            InvalidReason::SystemLocked => write!(f, "System was locked"),
            InvalidReason::ManuallyLocked => write!(f, "Session manually locked"),
            InvalidReason::LockGenerationMismatch { session_gen, current_gen } => {
                write!(f, "Lock generation mismatch (session: {}, current: {})", session_gen, current_gen)
            }
        }
    }
}

/// Serialize/deserialize helpers
impl DaemonRequest {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(json.into_bytes())
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl DaemonResponse {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(json.into_bytes())
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
