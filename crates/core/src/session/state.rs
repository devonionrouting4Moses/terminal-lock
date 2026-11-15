use crate::constants::{SESSION_TTL, INACTIVITY_TIMEOUT};
use crate::error::{Error, Result};
use crate::paths::Paths;
use crate::types::SessionInfo;
use chrono::Utc;
use super::store::SessionStore;

/// Session management - Each terminal has its own session
pub struct Session;

impl Session {
    /// Check if THIS terminal's session is valid
    /// IMPORTANT: Each terminal PID has its own session file
    /// Sessions have a FIXED 45-minute duration (no extension on command)
    /// Sessions expire after 15 minutes of inactivity
    pub fn is_valid(paths: &Paths) -> Result<bool> {
        let session_file = paths.session_file(); // Uses current PID
        
        if !session_file.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(&session_file)?;
        let session: SessionInfo = serde_json::from_str(&content)?;

        eprintln!("DEBUG: Session PID: {}", session.terminal_id);

        let now = Utc::now();
        
        // Check if session has expired (45-minute fixed duration)
        if now >= session.expiry {
            // Session expired - remove it
            let _ = std::fs::remove_file(&session_file);
            return Ok(false);
        }

        // Check if session has been inactive for 15 minutes
        let inactivity_duration = now.signed_duration_since(session.last_activity);
        if inactivity_duration >= chrono::Duration::from_std(INACTIVITY_TIMEOUT).unwrap() {
            // Inactive for 15+ minutes - expire session
            let _ = std::fs::remove_file(&session_file);
            return Ok(false);
        }

        // Session is valid - DO NOT update activity here
        // Activity is only updated by validate_and_extend() after commands
        Ok(true)
    }

    /// Create a new session for THIS terminal
    /// Cleans up any old session files before creating a new one
    /// CRITICAL FIX: Don't delete our OWN session file!
    pub fn create(paths: &Paths) -> Result<()> {
        eprintln!("DEBUG: Creating session...");
        eprintln!("DEBUG: Lock dir: {:?}", paths.lock_dir());
        
        // Ensure lock directory exists
        paths.ensure_lock_dir()?;

        // Get the session file path to know which PID we're using
        let session_file = paths.session_file();
        let session_filename = session_file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        eprintln!("DEBUG: Session file: {}", session_filename);

        // Extract our terminal ID from paths (this is the parent PID)
        // paths.session_file() returns something like "session.12345.json"
        let our_terminal_id = session_filename
            .strip_prefix("session.")
            .and_then(|s| s.strip_suffix(".json"))
            .unwrap_or("unknown")
            .to_string();
        eprintln!("DEBUG: Our terminal ID: {}", our_terminal_id);
        
        // Clean up any old session files (from previous shell instances)
        // This prevents accumulation of session files when sourcing shell config
        SessionStore::cleanup_old_sessions(paths, &session_filename)?;
        
        let now = Utc::now();
        let expiry = now + chrono::Duration::from_std(SESSION_TTL).unwrap();
        
        // Read current lock generation to include in session
        let lock_generation = Self::get_lock_generation(paths).unwrap_or(0);
        
        let session = SessionInfo {
            expiry,
            last_activity: now,
            terminal_id: our_terminal_id.clone(),
            lock_generation,
        };

        let session_file = paths.session_file();
        eprintln!("DEBUG: Session file path: {:?}", session_file);
        eprintln!("DEBUG: Session PID: {}", our_terminal_id);
        eprintln!("DEBUG: Lock generation: {}", lock_generation);
        
        let content = serde_json::to_string_pretty(&session)?;
        
        // Atomic write: Write to temp file first, then rename
        let temp_file = session_file.with_extension("tmp");
        std::fs::write(&temp_file, &content)?;
        
        // Atomic rename ensures file is complete when visible
        std::fs::rename(&temp_file, &session_file)?;
        
        // Force sync to disk to ensure session is persisted
        if let Ok(file) = std::fs::File::open(&session_file) {
            let _ = file.sync_all();
        }
        
        eprintln!("DEBUG: Session file created successfully");
        eprintln!("DEBUG: Verifying session file exists: {}", session_file.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &session_file,
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        // CRITICAL: Verify session was actually created by reading it back
        match std::fs::read_to_string(&session_file) {
            Ok(verify_content) => {
                match serde_json::from_str::<SessionInfo>(&verify_content) {
                    Ok(verify_session) => {
                        eprintln!("DEBUG: ✅ Session verified - terminal_id: {}, expiry: {}", 
                            verify_session.terminal_id, verify_session.expiry);
                    }
                    Err(e) => {
                        eprintln!("DEBUG: ❌ Session file exists but parse failed: {}", e);
                        return Err(Error::Unknown(format!("Session parse failed: {}", e)));
                    }
                }
            }
            Err(e) => {
                eprintln!("DEBUG: ❌ Session file read failed: {}", e);
                return Err(Error::Unknown(format!("Session read failed: {}", e)));
            }
        }

        // Create activity file with current timestamp
        Self::update_last_activity(paths)?;

        Ok(())
    }

    /// Validate session and update activity (but DO NOT extend expiry)
    /// This is called after each command to update activity timestamp
    /// The session has a FIXED 15-minute expiry that NEVER extends
    pub fn validate_and_extend(paths: &Paths) -> Result<bool> {
        let session_file = paths.session_file();
        
        if !session_file.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(&session_file)?;
        let session: SessionInfo = serde_json::from_str(&content)?;

        let now = Utc::now();
        
        // Check if session has expired (15-minute FIXED duration - NO EXTENSION)
        if now >= session.expiry {
            // Session expired - remove it
            let _ = std::fs::remove_file(&session_file);
            return Ok(false);
        }

        // Check if session has been inactive for 5 minutes
        let inactivity_duration = now.signed_duration_since(session.last_activity);
        if inactivity_duration >= chrono::Duration::from_std(INACTIVITY_TIMEOUT).unwrap() {
            // Inactive for 5+ minutes - expire session
            let _ = std::fs::remove_file(&session_file);
            return Ok(false);
        }

        // Session is valid - update ONLY the activity timestamp (NOT expiry)
        let updated_session = SessionInfo {
            expiry: session.expiry,           // Keep original expiry (15-minute hard limit)
            last_activity: now,               // Update activity timestamp
            terminal_id: session.terminal_id,
            lock_generation: session.lock_generation, // Keep original lock generation
        };
        
        let content = serde_json::to_string_pretty(&updated_session)?;
        std::fs::write(&session_file, content)?;
        Ok(true)
    }

    /// Remove THIS terminal's session
    pub fn remove(paths: &Paths) -> Result<()> {
        let session_file = paths.session_file();
        if session_file.exists() {
            std::fs::remove_file(session_file)?;
        }
        Ok(())
    }

    /// Check if this is a NEW terminal (no session exists for this PID)
    pub fn is_new_terminal(paths: &Paths) -> bool {
        !paths.session_file().exists()
    }

    /// Get current lock generation counter
    pub fn get_lock_generation(paths: &Paths) -> Result<u64> {
        let lock_gen_file = paths.lock_dir().join("lock_generation");
        
        if !lock_gen_file.exists() {
            return Ok(0);
        }
        
        let content = std::fs::read_to_string(&lock_gen_file)?;
        let gen: u64 = content.trim().parse()
            .map_err(|_| Error::Unknown("Invalid lock generation".into()))?;
        
        Ok(gen)
    }

    /// Increment lock generation counter (called when system locks)
    pub fn increment_lock_generation(paths: &Paths) -> Result<()> {
        let lock_gen_file = paths.lock_dir().join("lock_generation");
        let current = Self::get_lock_generation(paths).unwrap_or(0);
        let next = current + 1;
        
        paths.ensure_lock_dir()?;
        std::fs::write(&lock_gen_file, next.to_string())?;
        
        eprintln!("DEBUG: Lock generation incremented to {}", next);
        Ok(())
    }

    /// Check if session is valid including lock generation check
    pub fn is_valid_with_lock_check(paths: &Paths) -> Result<bool> {
        eprintln!("DEBUG: is_valid_with_lock_check called");

        // First check basic validity
        if !Self::is_valid(paths)? {
            eprintln!("DEBUG: Basic validity check failed");
            return Ok(false);
        }
        
        let session_file = paths.session_file();
        if !session_file.exists() {
            eprintln!("DEBUG: Session file doesn't exist after basic check");
            return Ok(false);
        }

        let content = std::fs::read_to_string(&session_file)?;
        let session: SessionInfo = serde_json::from_str(&content)?;
        
        eprintln!("DEBUG: Session file exists");
        eprintln!("DEBUG: Session lock generation: {}", session.lock_generation);
        eprintln!("DEBUG: Current lock generation: {}", Self::get_lock_generation(paths).unwrap_or(0));
        
        // Check if session was created before a lock event
        let current_lock_gen = Self::get_lock_generation(paths).unwrap_or(0);
        eprintln!("DEBUG: Session lock_generation: {}, Current: {}", 
            session.lock_generation, current_lock_gen);

        if session.lock_generation < current_lock_gen {
            // Session is from before the last lock - invalidate it
            let _ = std::fs::remove_file(&session_file);
            eprintln!("DEBUG: Session invalidated due to lock (gen {} < {})", 
                session.lock_generation, current_lock_gen);
            return Ok(false);
        }
        
        eprintln!("DEBUG: Session is valid with lock check passed");
        Ok(true)
    }

    /// Update the last activity timestamp for this terminal
    fn update_last_activity(paths: &Paths) -> Result<()> {
        let activity_file = paths.activity_file();
        let now = Utc::now();
        
        std::fs::write(&activity_file, now.to_rfc3339())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &activity_file,
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        Ok(())
    }
}
