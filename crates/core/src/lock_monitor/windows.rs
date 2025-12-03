//! Windows system lock detection
//! Monitors workstation lock status using WMI and system commands
//! Works with all Windows versions
//! Polling-based detection every 3 seconds

use crate::error::Result;
use crate::paths::Paths;
use std::process::Command;
use std::time::Duration;
use std::thread;

/// Start monitoring for system lock events on Windows
/// Uses WMI and system commands to detect workstation lock status
pub fn start_monitor(paths: &Paths) -> Result<()> {
    let paths = paths.clone();

    // Spawn background thread for lock monitoring
    thread::spawn(move || {
        let mut last_lock_state = false;

        loop {
            // Check if system is locked
            let is_locked = is_system_locked_windows();

            // If state changed from unlocked to locked
            if is_locked && !last_lock_state {
                eprintln!("🔒 Windows system lock detected (workstation locked)");
                if let Err(e) = super::invalidate_all_sessions(&paths) {
                    eprintln!("⚠️  Failed to invalidate sessions: {}", e);
                }
            }

            last_lock_state = is_locked;

            // Check every 3 seconds
            thread::sleep(Duration::from_secs(3));
        }
    });

    Ok(())
}

/// Check if system is locked on Windows
/// Uses multiple methods to detect workstation lock status
/// Returns true if system is locked
fn is_system_locked_windows() -> bool {
    // Method 1: Check via query session for disconnected sessions
    if let Ok(output) = Command::new("query")
        .args(&["session"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Look for "Disc" state which indicates disconnected/locked session
            if stdout.contains("Disc") {
                return true;
            }
        }
    }

    // Method 2: Check via wmic for session lock status
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "Win32_LogicalSessionConnectInfo", "get", "Dependent"])
        .output()
    {
        if !output.status.success() {
            // If we can't query, system might be locked
            return true;
        }
    }

    // Method 3: Check via tasklist (process list)
    // If we can't list processes, system might be locked
    if let Ok(output) = Command::new("tasklist").output() {
        if !output.status.success() {
            return true;
        }
    }

    false
}
