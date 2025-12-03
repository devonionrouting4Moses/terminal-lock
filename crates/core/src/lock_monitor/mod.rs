//! System lock detection module
//! Monitors for system lock/unlock events across different platforms
//! 
//! Platform-specific implementations:
//! - Linux: D-Bus event-driven detection (real-time)
//! - macOS: AppleScript screensaver detection (polling)
//! - Windows: WMI workstation lock detection (polling)

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::error::Result;
use crate::paths::Paths;
use std::thread;

/// Lock monitor that handles all platforms
pub struct LockMonitor;

impl LockMonitor {
    /// Start monitoring for system lock events
    /// Spawns a background thread that listens for lock events
    /// Each platform uses its native detection mechanism
    pub fn start(paths: &Paths) -> Result<()> {
        let paths = paths.clone();

        // Spawn background thread for lock monitoring
        thread::spawn(move || {
            #[cfg(target_os = "linux")]
            {
                eprintln!("🔵 Starting Linux D-Bus lock monitor...");
                if let Err(e) = linux::start_monitor(&paths) {
                    eprintln!("🔴 Linux lock monitor error: {}", e);
                }
            }

            #[cfg(target_os = "macos")]
            {
                eprintln!("🔵 Starting macOS lock monitor...");
                if let Err(e) = macos::start_monitor(&paths) {
                    eprintln!("🔴 macOS lock monitor error: {}", e);
                }
            }

            #[cfg(target_os = "windows")]
            {
                eprintln!("🔵 Starting Windows lock monitor...");
                if let Err(e) = windows::start_monitor(&paths) {
                    eprintln!("🔴 Windows lock monitor error: {}", e);
                }
            }
        });

        Ok(())
    }
}

/// Invalidate all sessions when system is locked
/// Called by platform-specific monitors when lock is detected
/// Deletes all session files so next terminal interaction requires re-authentication
pub(crate) fn invalidate_all_sessions(paths: &Paths) -> Result<()> {
    eprintln!("🔒 System lock detected - invalidating all sessions");

    if let Ok(session_files) = paths.all_session_files() {
        for session_file in session_files {
            if let Err(e) = std::fs::remove_file(&session_file) {
                eprintln!("⚠️  Failed to delete session file: {}", e);
            } else {
                eprintln!("✓ Deleted session: {:?}", session_file);
            }
        }
    }

    Ok(())
}
