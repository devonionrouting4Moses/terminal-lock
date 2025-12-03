//! macOS system lock detection
//! Detects actual system lock state (not just screensaver)
//! Uses multiple methods: Python/Quartz, AppleScript, screensaver
//! Works with all macOS versions (10.13+)
//! Polling-based detection every 2 seconds

use crate::error::Result;
use crate::paths::Paths;
use std::process::Command;
use std::time::Duration;
use std::thread;

/// Start monitoring for system lock events on macOS
/// Uses multiple detection methods for reliability
pub fn start_monitor(paths: &Paths) -> Result<()> {
    let paths = paths.clone();

    // Spawn background thread for lock monitoring
    thread::spawn(move || {
        let mut was_locked = false;

        loop {
            // Check if system is locked using multiple methods
            let is_locked = is_system_locked_macos();

            // Detect lock state change (unlocked → locked)
            if is_locked && !was_locked {
                eprintln!("🔒 macOS system locked - invalidating sessions");
                if let Err(e) = super::invalidate_all_sessions(&paths) {
                    eprintln!("⚠️  Failed to invalidate sessions: {}", e);
                }
                was_locked = true;
            } else if !is_locked && was_locked {
                eprintln!("🔓 macOS system unlocked");
                was_locked = false;
            }

            // Check every 2 seconds (macOS is responsive)
            thread::sleep(Duration::from_secs(2));
        }
    });

    Ok(())
}

/// Detect if macOS is locked using multiple methods
/// Returns true if system is locked (requires password to unlock)
fn is_system_locked_macos() -> bool {
    // Method 1: Python + Quartz (MOST RELIABLE)
    // Checks CGSSessionScreenIsLocked directly
    if check_cgession_python() {
        return true;
    }

    // Method 2: Login window check (RELIABLE)
    // If loginwindow is frontmost app, system is locked
    if check_login_window() {
        return true;
    }

    // Method 3: Screen saver check (FALLBACK)
    // Screensaver running usually means locked
    if check_screensaver() {
        return true;
    }

    false
}

/// Check lock status using Python + Quartz framework
/// This is the most reliable method for detecting actual lock state
fn check_cgession_python() -> bool {
    if let Ok(output) = Command::new("python3")
        .args(&[
            "-c",
            r#"import Quartz; d = Quartz.CGSessionCopyCurrentDictionary(); print('1' if d and d.get('CGSSessionScreenIsLocked', 0) == 1 else '0')"#,
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.trim() == "1";
        }
    }
    false
}

/// Check if login window is the frontmost application
/// If loginwindow is frontmost, system is locked
fn check_login_window() -> bool {
    let script = r#"tell application "System Events"
    try
        set frontApp to name of first application process whose frontmost is true
        if frontApp is "loginwindow" then
            return "true"
        else
            return "false"
        end if
    on error
        return "false"
    end try
end tell"#;

    if let Ok(output) = Command::new("osascript")
        .args(&["-e", script])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.trim() == "true";
        }
    }
    false
}

/// Check if screensaver is running
/// Screensaver running usually indicates system is locked
fn check_screensaver() -> bool {
    if let Ok(output) = Command::new("osascript")
        .args(&[
            "-e",
            "tell application \"System Events\" to get screen saver running",
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.trim() == "true";
        }
    }
    false
}
