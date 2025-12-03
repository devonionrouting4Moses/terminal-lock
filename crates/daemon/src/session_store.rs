use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;
use uuid::Uuid;
use terminal_lock_ipc::InvalidReason;
use terminal_lock_core::constants::SESSION_TTL;

/// Session state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Paused,
    Expired,
    Locked,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Active => "Active",
            SessionState::Paused => "Paused",
            SessionState::Expired => "Expired",
            SessionState::Locked => "Locked",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Paused" => SessionState::Paused,
            "Expired" => SessionState::Expired,
            "Locked" => SessionState::Locked,
            _ => SessionState::Active,
        }
    }
}

impl From<SessionState> for terminal_lock_ipc::SessionState {
    fn from(state: SessionState) -> Self {
        match state {
            SessionState::Active => terminal_lock_ipc::SessionState::Active,
            SessionState::Paused => terminal_lock_ipc::SessionState::Paused,
            SessionState::Expired => terminal_lock_ipc::SessionState::Expired,
            SessionState::Locked => terminal_lock_ipc::SessionState::Locked,
        }
    }
}

/// Stored session in database
#[derive(Debug, Clone)]
pub struct StoredSession {
    pub id: String,
    pub terminal_pid: u32,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub last_activity: SystemTime,
    pub state: SessionState,
    pub lock_generation: u64,
}

/// Session store with SQLite backend
pub struct SessionStore {
    db_path: PathBuf,
    conn: Mutex<Connection>,
}

impl SessionStore {
    /// Create new session store with SQLite backend
    pub fn new(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(&db_path)?;
        
        // Initialize database schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                terminal_pid INTEGER NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                last_activity INTEGER NOT NULL,
                state TEXT NOT NULL,
                lock_generation INTEGER NOT NULL,
                terminal_path TEXT
            );
            
            CREATE TABLE IF NOT EXISTS attempts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                success BOOLEAN NOT NULL,
                shell_pid INTEGER,
                ip_address TEXT,
                failure_reason TEXT
            );
            
            CREATE TABLE IF NOT EXISTS lockouts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                shell_pid INTEGER NOT NULL,
                locked_at INTEGER NOT NULL,
                unlocks_at INTEGER NOT NULL,
                reason TEXT NOT NULL,
                lockout_type TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                shell_pid INTEGER,
                session_id TEXT,
                details TEXT
            );
            
            CREATE TABLE IF NOT EXISTS system_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_sessions_state ON sessions(state);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
            CREATE INDEX IF NOT EXISTS idx_sessions_pid ON sessions(terminal_pid);
            CREATE INDEX IF NOT EXISTS idx_attempts_timestamp ON attempts(timestamp);
            CREATE INDEX IF NOT EXISTS idx_attempts_user ON attempts(user_id);
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
            CREATE INDEX IF NOT EXISTS idx_audit_event ON audit_log(event_type);
            
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA foreign_keys = ON;",
        )?;
        
        Ok(SessionStore {
            db_path,
            conn: Mutex::new(conn),
        })
    }

    /// Create a new session for a terminal
    pub fn create_session(
        &self,
        terminal_pid: u32,
        _password_hash: &str,
    ) -> Result<(String, SystemTime), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now();
        let expires_at = now + SESSION_TTL;
        
        let now_secs = now.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        let expires_secs = expires_at.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        
        // Get current user ID
        let user_id = unsafe { libc::getuid() } as i32;
        
        // Get terminal path if available
        let terminal_path = std::env::var("TTY").ok();
        
        conn.execute(
            "INSERT INTO sessions (id, user_id, terminal_pid, created_at, expires_at, last_activity, state, lock_generation, terminal_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &session_id,
                user_id,
                terminal_pid as i32,
                now_secs,
                expires_secs,
                now_secs,
                SessionState::Active.as_str(),
                0i64,
                terminal_path,
            ],
        )?;
        
        Ok((session_id, expires_at))
    }

    /// Get session by ID
    pub fn get_session(&self, terminal_pid: u32) -> Result<StoredSession, InvalidReason> {
        let conn = self.conn.lock().map_err(|_| InvalidReason::NotFound)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, terminal_pid, created_at, expires_at, last_activity, state, lock_generation
             FROM sessions WHERE terminal_pid = ?1"
        ).map_err(|_| InvalidReason::NotFound)?;
        
        let session = stmt.query_row(params![terminal_pid as i32], |row| {
            let created_secs = row.get::<_, i64>(2)?;
            let expires_secs = row.get::<_, i64>(3)?;
            let activity_secs = row.get::<_, i64>(4)?;
            let state_str: String = row.get(5)?;
            
            Ok(StoredSession {
                id: row.get(0)?,
                terminal_pid: row.get::<_, i32>(1)? as u32,
                created_at: std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_secs as u64),
                expires_at: std::time::UNIX_EPOCH + std::time::Duration::from_secs(expires_secs as u64),
                last_activity: std::time::UNIX_EPOCH + std::time::Duration::from_secs(activity_secs as u64),
                state: SessionState::from_str(&state_str),
                lock_generation: row.get::<_, i64>(6)? as u64,
            })
        }).optional().map_err(|_| InvalidReason::NotFound)?;
        
        session.ok_or(InvalidReason::NotFound)
    }

    /// Validate session for terminal PID
    pub fn validate_session(&self, terminal_pid: u32) -> Result<(String, u64, u64), InvalidReason> {
        let session = self.get_session(terminal_pid)?;
        let now = SystemTime::now();
        
        // Check if expired
        if now >= session.expires_at {
            let _ = self.invalidate_session(terminal_pid);
            return Err(InvalidReason::Expired);
        }
        
        // Check if paused (system locked)
        if session.state == SessionState::Paused {
            return Err(InvalidReason::SystemLocked);
        }
        
        // Check if manually locked
        if session.state == SessionState::Locked {
            return Err(InvalidReason::ManuallyLocked);
        }
        
        // Calculate time remaining
        let time_remaining = session.expires_at
            .duration_since(now)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        Ok((session.id, time_remaining, session.lock_generation))
    }

    /// Extend session activity timestamp
    pub fn extend_session(&self, terminal_pid: u32) -> Result<String, InvalidReason> {
        let conn = self.conn.lock().map_err(|_| InvalidReason::NotFound)?;
        let now = SystemTime::now();
        let now_secs = now.duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .map_err(|_| InvalidReason::NotFound)?;
        
        conn.execute(
            "UPDATE sessions SET last_activity = ?1 WHERE terminal_pid = ?2",
            params![now_secs, terminal_pid as i32],
        ).map_err(|_| InvalidReason::NotFound)?;
        
        // Get session ID
        let mut stmt = conn.prepare("SELECT id FROM sessions WHERE terminal_pid = ?1")
            .map_err(|_| InvalidReason::NotFound)?;
        
        let session_id: String = stmt.query_row(params![terminal_pid as i32], |row| row.get(0))
            .map_err(|_| InvalidReason::NotFound)?;
        
        Ok(session_id)
    }

    /// Invalidate session for terminal PID
    pub fn invalidate_session(&self, terminal_pid: u32) -> Result<(), InvalidReason> {
        let conn = self.conn.lock().map_err(|_| InvalidReason::NotFound)?;
        
        conn.execute(
            "DELETE FROM sessions WHERE terminal_pid = ?1",
            params![terminal_pid as i32],
        ).map_err(|_| InvalidReason::NotFound)?;
        
        Ok(())
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> Result<Vec<StoredSession>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, terminal_pid, created_at, expires_at, last_activity, state, lock_generation
             FROM sessions WHERE state = 'Active' ORDER BY created_at DESC"
        )?;
        
        let sessions = stmt.query_map([], |row| {
            let created_secs = row.get::<_, i64>(2)?;
            let expires_secs = row.get::<_, i64>(3)?;
            let activity_secs = row.get::<_, i64>(4)?;
            let state_str: String = row.get(5)?;
            
            Ok(StoredSession {
                id: row.get(0)?,
                terminal_pid: row.get::<_, i32>(1)? as u32,
                created_at: std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_secs as u64),
                expires_at: std::time::UNIX_EPOCH + std::time::Duration::from_secs(expires_secs as u64),
                last_activity: std::time::UNIX_EPOCH + std::time::Duration::from_secs(activity_secs as u64),
                state: SessionState::from_str(&state_str),
                lock_generation: row.get::<_, i64>(6)? as u64,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;
        
        Ok(sessions)
    }

    /// Pause all active sessions (system locked)
    pub fn pause_all_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE sessions SET state = ?1 WHERE state = ?2",
            params![SessionState::Paused.as_str(), SessionState::Active.as_str()],
        )?;
        Ok(())
    }

    /// Resume all paused sessions (system unlocked)
    pub fn resume_all_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE sessions SET state = ?1 WHERE state = ?2",
            params![SessionState::Active.as_str(), SessionState::Paused.as_str()],
        )?;
        Ok(())
    }

    /// Lock all sessions
    pub fn lock_all_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute(
            "UPDATE sessions SET state = ?1 WHERE state != ?2",
            params![SessionState::Locked.as_str(), SessionState::Locked.as_str()],
        )?;
        Ok(())
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let now = SystemTime::now();
        let now_secs = now.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
        
        conn.execute(
            "DELETE FROM sessions WHERE expires_at < ?1",
            params![now_secs],
        )?;
        
        Ok(conn.last_insert_rowid() as u32)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<(u32, u32, u32), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        
        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM sessions",
            [],
            |row| row.get(0),
        )?;
        
        let active: u32 = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE state = ?1",
            params![SessionState::Active.as_str()],
            |row| row.get(0),
        )?;
        
        let paused: u32 = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE state = ?1",
            params![SessionState::Paused.as_str()],
            |row| row.get(0),
        )?;
        
        Ok((total, active, paused))
    }

    /// Log authentication attempt
    pub fn log_attempt(
        &self,
        user_id: i32,
        success: bool,
        shell_pid: Option<u32>,
        failure_reason: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        
        conn.execute(
            "INSERT INTO attempts (user_id, timestamp, success, shell_pid, failure_reason)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                user_id,
                now,
                success,
                shell_pid.map(|p| p as i32),
                failure_reason,
            ],
        )?;
        
        Ok(())
    }

    /// Log audit event
    pub fn log_audit(
        &self,
        event_type: &str,
        user_id: i32,
        shell_pid: Option<u32>,
        session_id: Option<&str>,
        details: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        
        conn.execute(
            "INSERT INTO audit_log (timestamp, event_type, user_id, shell_pid, session_id, details)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                now,
                event_type,
                user_id,
                shell_pid.map(|p| p as i32),
                session_id,
                details,
            ],
        )?;
        
        Ok(())
    }

    /// Create lockout record
    pub fn create_lockout(
        &self,
        user_id: i32,
        shell_pid: u32,
        duration_secs: i64,
        reason: &str,
        lockout_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let unlocks_at = now + duration_secs;
        
        conn.execute(
            "INSERT INTO lockouts (user_id, shell_pid, locked_at, unlocks_at, reason, lockout_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user_id,
                shell_pid as i32,
                now,
                unlocks_at,
                reason,
                lockout_type,
            ],
        )?;
        
        Ok(())
    }
}
