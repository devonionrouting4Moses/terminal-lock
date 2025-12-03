/// Failed authentication attempts tracking and lockout management
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use terminal_lock_core::constants::*;
use terminal_lock_core::error::Result;
use terminal_lock_core::Paths;

/// Attempt phases: 3 phases with 3 attempts each (total 9)
/// Phase 1 (attempts 1-3): Warning
/// Phase 2 (attempts 4-6): Increasing delay
/// Phase 3 (attempts 7-9): System lock
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptPhase {
    /// Phase 1: Attempts 1-3 (warning phase)
    Warning,
    /// Phase 2: Attempts 4-6 (delay phase)
    Delay,
    /// Phase 3: Attempts 7-9 (lock phase)
    Lock,
}

impl AttemptPhase {
    /// Get phase from attempt count
    pub fn from_attempts(attempts: u32) -> Self {
        match attempts {
            1..=3 => AttemptPhase::Warning,
            4..=6 => AttemptPhase::Delay,
            7..=9 => AttemptPhase::Lock,
            _ => AttemptPhase::Lock, // After MAX_TOTAL_ATTEMPTS, stay in lock phase
        }
    }

    /// Get delay in seconds for this phase
    pub fn delay_seconds(&self) -> u64 {
        match self {
            AttemptPhase::Warning => 0,      // No delay
            AttemptPhase::Delay => 5,        // 5 seconds between attempts
            AttemptPhase::Lock => LOCKOUT_DURATION.as_secs(),      // Lockout duration
        }
    }

    /// Get user-friendly message for this phase
    pub fn message(&self, attempts: u32) -> String {
        match self {
            AttemptPhase::Warning => {
                match attempts {
                    1 => "⚠️  Warning: 1 failed attempt. Please try again.".to_string(),
                    2 => "⚠️  Warning: 2 failed attempts. Please try again.".to_string(),
                    3 => "⚠️  Warning: 3 failed attempts. Final warning!".to_string(),
                    _ => format!("⚠️  Warning: {} failed attempt(s).", attempts),
                }
            }
            AttemptPhase::Delay => {
                let remaining = MAX_TOTAL_ATTEMPTS - attempts;
                match attempts {
                    4 => format!("⏱️  Too many attempts. {} remaining. Please try again.", MAX_TOTAL_ATTEMPTS - 4),
                    5 => format!("⏱️  Too many attempts. {} remaining. Please try again.", MAX_TOTAL_ATTEMPTS - 5),
                    6 => format!("⏱️  Too many attempts. {} remaining. Final chance!", MAX_TOTAL_ATTEMPTS - 6),
                    _ => format!("⏱️  Too many attempts. {} remaining.", remaining),
                }
            }
            AttemptPhase::Lock => {
                match attempts {
                    7 => format!("🔒 Critical phase: {} attempts remaining before system lock.", MAX_TOTAL_ATTEMPTS - 7),
                    8 => format!("🔒 Critical phase: {} attempts remaining before system lock.", MAX_TOTAL_ATTEMPTS - 8),
                    9 => "🔒 System locked due to too many failed attempts.".to_string(),
                    _ => "🔒 Critical phase.".to_string(),
                }
            }
        }
    }
}

/// Information about authentication attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptsInfo {
    /// Total failed attempts across all sessions
    pub total: u32,
    /// Failed attempts in current session
    pub session: u32,
    /// Timestamp of last failed attempt
    pub last_attempt: u64,
    /// Timestamp when attempts counter was reset
    pub reset_at: u64,
    /// Current phase (1-3)
    pub phase: u32,
}

impl Default for AttemptsInfo {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            total: 0,
            session: 0,
            last_attempt: now,
            reset_at: now,
            phase: 1,
        }
    }
}

impl AttemptsInfo {
    /// Get current phase
    pub fn get_phase(&self) -> AttemptPhase {
        AttemptPhase::from_attempts(self.total)
    }

    /// Check if in lock phase (phase 3)
    pub fn is_locked(&self) -> bool {
        self.get_phase() == AttemptPhase::Lock
    }

    /// Get remaining attempts before lock
    pub fn remaining_attempts(&self) -> u32 {
        if self.total >= MAX_TOTAL_ATTEMPTS {
            0
        } else {
            MAX_TOTAL_ATTEMPTS - self.total
        }
    }

    /// Get delay for current phase
    pub fn delay_seconds(&self) -> u64 {
        self.get_phase().delay_seconds()
    }

    /// Get user message for current state
    pub fn message(&self) -> String {
        self.get_phase().message(self.total)
    }
}

/// Tracks failed authentication attempts
pub struct AttemptsTracker;

impl AttemptsTracker {
    /// Get attempts file path
    fn attempts_file(paths: &Paths) -> PathBuf {
        paths.lock_dir().join("attempts.json")
    }

    /// Load current attempts info
    pub fn load(paths: &Paths) -> Result<AttemptsInfo> {
        let file_path = Self::attempts_file(paths);
        
        if !file_path.exists() {
            return Ok(AttemptsInfo::default());
        }
        
        let content = fs::read_to_string(&file_path)?;
        let info: AttemptsInfo = serde_json::from_str(&content)?;
        Ok(info)
    }

    /// Save attempts info
    fn save(paths: &Paths, info: &AttemptsInfo) -> Result<()> {
        let file_path = Self::attempts_file(paths);
        
        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(info)?;
        fs::write(&file_path, content)?;
        Ok(())
    }

    /// Increment failed attempts counter and return phase info
    /// Returns: (total_attempts, phase, delay_seconds, message)
    pub fn increment(paths: &Paths) -> Result<(u32, AttemptPhase, u64, String)> {
        let mut info = Self::load(paths)?;
        
        // Don't increment if already at MAX_TOTAL_ATTEMPTS (locked)
        if info.total < MAX_TOTAL_ATTEMPTS {
            info.total += 1;
            info.session += 1;
        }
        
        info.last_attempt = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        info.phase = match info.get_phase() {
            AttemptPhase::Warning => 1,
            AttemptPhase::Delay => 2,
            AttemptPhase::Lock => 3,
        };
        
        Self::save(paths, &info)?;
        
        let phase = info.get_phase();
        let delay = phase.delay_seconds();
        let message = info.message();
        
        Ok((info.total, phase, delay, message))
    }

    /// Get current attempt info without incrementing
    pub fn get_info(paths: &Paths) -> Result<(u32, AttemptPhase, u64, String)> {
        let info = Self::load(paths)?;
        let phase = info.get_phase();
        let delay = phase.delay_seconds();
        let message = info.message();
        
        Ok((info.total, phase, delay, message))
    }

    /// Check if should apply delay before next attempt
    pub fn should_delay(paths: &Paths) -> Result<Option<u64>> {
        let info = Self::load(paths)?;
        let phase = info.get_phase();
        
        match phase {
            AttemptPhase::Warning => Ok(None),
            AttemptPhase::Delay | AttemptPhase::Lock => {
                Ok(Some(phase.delay_seconds()))
            }
        }
    }

    /// Reset attempts counter
    pub fn reset(paths: &Paths) -> Result<()> {
        let mut info = AttemptsInfo::default();
        info.reset_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self::save(paths, &info)?;
        Ok(())
    }

    /// Check if laptop was unlocked (resets counter)
    pub fn check_laptop_unlock(paths: &Paths) -> Result<bool> {
        // Check if system lock marker file exists
        let lock_marker = paths.lock_marker_file();
        
        if lock_marker.exists() {
            // System was locked, now it's unlocked
            fs::remove_file(&lock_marker)?;
            Self::reset(paths)?;
            return Ok(true);
        }
        
        Ok(false)
    }

    /// Mark system as locked
    pub fn mark_system_locked(paths: &Paths) -> Result<()> {
        let lock_marker = paths.lock_marker_file();
        fs::write(&lock_marker, "")?;
        Ok(())
    }

    /// Get time since last attempt (in seconds)
    pub fn time_since_last_attempt(paths: &Paths) -> Result<u64> {
        let info = Self::load(paths)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(now.saturating_sub(info.last_attempt))
    }

    /// Check if should reset attempts (DEPRECATED - DO NOT USE)
    /// Attempts are reset ONLY when the machine is unlocked after being locked.
    /// There is NO automatic time-based reset - use check_laptop_unlock() instead.
    #[deprecated(note = "Use check_laptop_unlock() instead for reset on machine unlock")]
    pub fn should_reset_attempts(paths: &Paths) -> Result<bool> {
        // This function is kept for backwards compatibility but should not be used
        // Attempts reset ONLY when machine is unlocked, not on any timer
        Ok(false)
    }

    /// Clear all attempts data
    pub fn clear_all(paths: &Paths) -> Result<()> {
        let file_path = Self::attempts_file(paths);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        
        let lock_marker = paths.lock_marker_file();
        if lock_marker.exists() {
            fs::remove_file(&lock_marker)?;
        }
        
        Ok(())
    }
}

/// Manages lockout periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutInfo {
    /// When lockout expires (Unix timestamp)
    pub expires_at: u64,
    /// Reason for lockout
    pub reason: String,
}

/// Lockout manager
pub struct LockoutManager;

impl LockoutManager {
    /// Get lockout file path
    fn lockout_file(paths: &Paths) -> PathBuf {
        paths.lock_dir().join("lockout.json")
    }

    /// Check if currently locked out
    pub fn is_locked(paths: &Paths) -> Result<Option<u64>> {
        let file_path = Self::lockout_file(paths);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&file_path)?;
        let info: LockoutInfo = serde_json::from_str(&content)?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now < info.expires_at {
            Ok(Some(info.expires_at))
        } else {
            // Lockout expired, remove file
            fs::remove_file(&file_path).ok();
            Ok(None)
        }
    }

    /// Create a lockout
    pub fn create_lockout(paths: &Paths, duration_secs: u64, reason: &str) -> Result<u64> {
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + duration_secs;
        
        let info = LockoutInfo {
            expires_at,
            reason: reason.to_string(),
        };
        
        let file_path = Self::lockout_file(paths);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&info)?;
        fs::write(&file_path, content)?;
        
        Ok(expires_at)
    }

    /// Remove lockout
    pub fn remove_lockout(paths: &Paths) -> Result<()> {
        let file_path = Self::lockout_file(paths);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        Ok(())
    }

    /// Clean up all lockouts
    pub fn cleanup_all_lockouts(paths: &Paths) -> Result<()> {
        Self::remove_lockout(paths)?;
        AttemptsTracker::clear_all(paths)?;
        Ok(())
    }

    /// Get remaining lockout time (in seconds)
    pub fn remaining_lockout_time(paths: &Paths) -> Result<Option<u64>> {
        if let Some(expires_at) = Self::is_locked(paths)? {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            Ok(Some(expires_at.saturating_sub(now)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempts_info_default() {
        let info = AttemptsInfo::default();
        assert_eq!(info.total, 0);
        assert_eq!(info.session, 0);
    }

    #[test]
    fn test_lockout_info_creation() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let lockout_duration = LOCKOUT_DURATION.as_secs();
        let info = LockoutInfo {
            expires_at: now + lockout_duration,
            reason: "test".to_string(),
        };
        
        assert!(info.expires_at > now);
    }
}
