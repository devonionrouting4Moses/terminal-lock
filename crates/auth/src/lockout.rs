use chrono::{DateTime, Utc};
use terminal_lock_core::{
    constants::*,
    error::*,
    paths::Paths,
    types::{AttemptsInfo, LockoutInfo},
};

/// Manage lockout state
pub struct LockoutManager;

impl LockoutManager {
    /// Check if terminal is in lockout period
    pub fn is_locked(paths: &Paths) -> Result<Option<DateTime<Utc>>> {
        let lockout_file = paths.lockout_file();
        
        if !lockout_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&lockout_file)?;
        let lockout: LockoutInfo = serde_json::from_str(&content)?;

        let now = Utc::now();
        if now < lockout.until {
            Ok(Some(lockout.until))
        } else {
            // Lockout expired
            let _ = std::fs::remove_file(&lockout_file);
            Ok(None)
        }
    }

    /// Create lockout for this terminal
    pub fn create_lockout(paths: &Paths) -> Result<DateTime<Utc>> {
        let until = Utc::now() + chrono::Duration::from_std(LOCKOUT_DURATION).unwrap();
        let lockout = LockoutInfo {
            until,
            terminal_id: std::process::id().to_string(),
        };

        let content = serde_json::to_string_pretty(&lockout)?;
        std::fs::write(paths.lockout_file(), content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                paths.lockout_file(),
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        Ok(until)
    }

    /// Remove lockout
    pub fn remove_lockout(paths: &Paths) -> Result<()> {
        let lockout_file = paths.lockout_file();
        if lockout_file.exists() {
            std::fs::remove_file(lockout_file)?;
        }
        Ok(())
    }

    /// Clean up all lockouts
    pub fn cleanup_all_lockouts(paths: &Paths) -> Result<()> {
        for lockout_file in paths.all_lockout_files()? {
            let _ = std::fs::remove_file(lockout_file);
        }
        Ok(())
    }
}

/// Track authentication attempts
pub struct AttemptsTracker;

impl AttemptsTracker {
    /// Load attempts info
    pub fn load(paths: &Paths) -> Result<AttemptsInfo> {
        let file = paths.total_attempts_file();
        
        if !file.exists() {
            return Ok(AttemptsInfo::default());
        }

        let content = std::fs::read_to_string(&file)?;
        let info = serde_json::from_str(&content)?;
        Ok(info)
    }

    /// Save attempts info
    pub fn save(paths: &Paths, info: &AttemptsInfo) -> Result<()> {
        let content = serde_json::to_string_pretty(info)?;
        std::fs::write(paths.total_attempts_file(), content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                paths.total_attempts_file(),
                std::fs::Permissions::from_mode(0o600),
            )?;
        }

        Ok(())
    }

    /// Increment attempts counter
    pub fn increment(paths: &Paths) -> Result<u32> {
        let mut info = Self::load(paths)?;
        info.total += 1;
        Self::save(paths, &info)?;
        Ok(info.total)
    }

    /// Reset attempts counter
    pub fn reset(paths: &Paths) -> Result<()> {
        let info = AttemptsInfo::default();
        Self::save(paths, &info)?;
        Ok(())
    }

    /// Check if laptop was recently unlocked (resets counter)
    pub fn check_laptop_unlock(paths: &Paths) -> Result<bool> {
        let marker_file = paths.lock_marker_file();
        let mut should_reset = false;

        // Check if system lock marker exists (laptop was locked and now unlocked)
        if marker_file.exists() {
            should_reset = true;
            // Remove the marker file since we've detected the unlock
            let _ = std::fs::remove_file(&marker_file);
        }

        // Also check boot time for system reboots
        let current_boot_time = Self::get_boot_time()?;
        let mut info = Self::load(paths)?;

        // If boot time changed significantly (>5 min diff), system was rebooted
        let diff = (current_boot_time - info.last_boot_time).abs();
        if diff > 300 {
            should_reset = true;
        }

        if should_reset {
            info.total = 0;
            info.last_boot_time = current_boot_time;
            Self::save(paths, &info)?;
            LockoutManager::cleanup_all_lockouts(paths)?;
        }

        // Update last boot time
        info.last_boot_time = current_boot_time;
        Self::save(paths, &info)?;

        Ok(should_reset)
    }

    /// Get system boot time (Unix timestamp)
    #[cfg(target_os = "linux")]
    fn get_boot_time() -> Result<i64> {
        let uptime = std::fs::read_to_string("/proc/uptime")?;
        let uptime_secs: f64 = uptime
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Ok(now - uptime_secs as i64)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_boot_time() -> Result<i64> {
        // Fallback for non-Linux systems
        Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64)
    }
}