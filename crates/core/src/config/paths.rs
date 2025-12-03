use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Manages all file paths for terminal-lock
#[derive(Debug, Clone)]
pub struct Paths {
    lock_dir: PathBuf,
    terminal_id: String,
}

impl Paths {
    /// Create new Paths instance
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| Error::Unknown("Cannot find home directory".into()))?;
        let lock_dir = home.join(".terminal_lock");
        //let terminal_id = std::process::id().to_string();
        // Ensure lock directory exists
        // std::fs::create_dir_all(&lock_dir)?;
        
        // Use parent process ID (shell) instead of current process ID
        // This ensures that terminal-lock check/extend commands use the same session as terminal-lock auth
        // When called from shell hooks, the current process is a child of the shell
        // let terminal_id = if let Ok(ppid_str) = std::env::var("PPID") {
        //     // PPID environment variable might be set
        //     ppid_str
        // } else {
        //     // Fallback: Use current PID if PPID not available
        //     std::process::id().to_string()
        // };

        let terminal_id = Self::get_parent_pid().unwrap_or_else(|| std::process::id().to_string());

        Ok(Self { lock_dir, terminal_id })
    }
    
    /// Get parent process ID (shell PID)
    fn get_parent_pid() -> Option<String> {
        // Try to read /proc/self/stat to get PPID
        #[cfg(unix)]
        {
            if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
                // Format: pid (comm) state ppid ...
                // We need the 4th field (ppid)
                let parts: Vec<&str> = stat.split_whitespace().collect();
                if parts.len() > 3 {
                    return Some(parts[3].to_string());
                }
            }
        }
        None
    }

    /// Get the lock directory
    pub fn lock_dir(&self) -> &Path {
        &self.lock_dir
    }

    /// Ensure lock directory exists
    pub fn ensure_lock_dir(&self) -> Result<()> {
        if !self.lock_dir.exists() {
            std::fs::create_dir_all(&self.lock_dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&self.lock_dir, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        Ok(())
    }

    /// Password hash file
    pub fn pass_hash_file(&self) -> PathBuf {
        self.lock_dir.join("pass.hash")
    }

    /// Salt file
    pub fn salt_file(&self) -> PathBuf {
        self.lock_dir.join("salt")
    }

    /// Display name file
    pub fn name_file(&self) -> PathBuf {
        self.lock_dir.join("display_name")
    }

    /// Preferences file
    pub fn preferences_file(&self) -> PathBuf {
        self.lock_dir.join("preferences.json")
    }

    /// Session file (per-terminal)
    pub fn session_file(&self) -> PathBuf {
        self.lock_dir.join(format!("session.{}.json", self.terminal_id))
    }

    /// Lockout file (per-terminal)
    pub fn lockout_file(&self) -> PathBuf {
        self.lock_dir.join(format!("lockout.{}.json", self.terminal_id))
    }

    /// Total attempts file (global)
    pub fn total_attempts_file(&self) -> PathBuf {
        self.lock_dir.join("total_attempts.json")
    }

    /// Last unlock time file
    pub fn last_unlock_file(&self) -> PathBuf {
        self.lock_dir.join("last_unlock")
    }

    /// Animation PID file (per-terminal)
    pub fn animation_pid_file(&self) -> PathBuf {
        self.lock_dir.join(format!("animation.{}.pid", self.terminal_id))
    }

    /// Activity timestamp file (per-terminal) - tracks last command execution
    pub fn activity_file(&self) -> PathBuf {
        self.lock_dir.join(format!("activity.{}.txt", self.terminal_id))
    }

    /// System lock marker file - indicates laptop was locked
    pub fn lock_marker_file(&self) -> PathBuf {
        self.lock_dir.join("system_locked")
    }

    /// Get all session files
    pub fn all_session_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if self.lock_dir.exists() {
            for entry in std::fs::read_dir(&self.lock_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("session.") && name.ends_with(".json") {
                        files.push(path);
                    }
                }
            }
        }
        Ok(files)
    }

    /// Get all lockout files
    pub fn all_lockout_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if self.lock_dir.exists() {
            for entry in std::fs::read_dir(&self.lock_dir)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("lockout.") && name.ends_with(".json") {
                        files.push(path);
                    }
                }
            }
        }
        Ok(files)
    }
}

impl Default for Paths {
    fn default() -> Self {
        Self::new().expect("Failed to create Paths")
    }
}