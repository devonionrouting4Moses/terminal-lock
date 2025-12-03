//! Linux system lock detection via D-Bus
//! Listens for ScreenSaver signals from org.freedesktop.ScreenSaver
//! Works with GNOME, KDE, XFCE, and other desktop environments
//! Uses D-Bus for real-time event-driven detection

use crate::error::Result;
use crate::paths::Paths;
use std::process::Command;
use std::thread;

/// Start monitoring for system lock events on Linux via D-Bus
/// Listens for org.freedesktop.ScreenSaver.ActiveChanged signals
/// Falls back to polling if D-Bus is unavailable
pub fn start_monitor(paths: &Paths) -> Result<()> {
    let paths = paths.clone();

    // Spawn background thread for lock monitoring
    thread::spawn(move || {
        // First try D-Bus event-driven monitoring
        if verify_dbus_available() {
            eprintln!("🔵 Starting D-Bus lock monitor...");
            if let Err(e) = monitor_dbus_signals(&paths) {
                eprintln!("⚠️  D-Bus monitoring failed: {}", e);
                eprintln!("📍 Falling back to polling-based detection");
                // Fall back to polling
                if let Err(e) = monitor_polling(&paths) {
                    eprintln!("🔴 Polling monitor also failed: {}", e);
                }
            }
        } else {
            eprintln!("⚠️  D-Bus unavailable");
            eprintln!("📍 Using polling-based lock detection");
            // Use polling-based detection
            if let Err(e) = monitor_polling(&paths) {
                eprintln!("🔴 Polling monitor failed: {}", e);
            }
        }
    });

    Ok(())
}

/// Monitor D-Bus for lock signals using dbus-monitor
/// This is event-driven - signals are received in real-time
/// Listens for org.gnome.SessionManager.Presence.StatusChanged signals
/// Status values: 0=available, 1=invisible, 2=busy, 3=idle
fn monitor_dbus_signals(paths: &Paths) -> Result<()> {
    // Start dbus-monitor to listen for GNOME SessionManager Presence signals
    let mut child = std::process::Command::new("dbus-monitor")
        .args(&[
            "--session",
            "type='signal',interface='org.gnome.SessionManager.Presence',member='StatusChanged'",
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| crate::error::Error::Unknown(format!("Failed to start dbus-monitor: {}", e)))?;

    let stdout = child.stdout.take()
        .ok_or_else(|| crate::error::Error::Unknown("Failed to get dbus-monitor stdout".into()))?;

    let reader = std::io::BufReader::new(stdout);
    use std::io::BufRead;

    // Track previous state to detect transitions
    let mut was_locked = false;

    // Read signals from dbus-monitor
    for line in reader.lines() {
        if let Ok(line) = line {
            // Look for StatusChanged signal with uint32 values
            // Status 0 = available (unlocked)
            // Status 1 = invisible (locked/away)
            // Status 2 = busy
            // Status 3 = idle
            // We want to invalidate sessions when status changes to anything other than 0 (available)
            // || line.contains("uint32 2") || line.contains("uint32 3") 
            if line.contains("uint32 1"){
                eprintln!("🔒 System lock detected via D-Bus (GNOME SessionManager Presence)");
                // Increment lock generation to invalidate all sessions from before the lock
                use crate::session::Session;
                let _ = Session::increment_lock_generation(paths);
                super::invalidate_all_sessions(paths)?;
                was_locked = true;
            } else if line.contains("uint32 0") {
                // System unlocked (status = 0 = available)
                if was_locked {
                    eprintln!("🔓 System unlocked - sessions remain invalid until re-authentication");
                    was_locked = false;
                }
            }
        }
    }

    Ok(())
}

/// Verify D-Bus is available and SessionManager Presence interface exists
pub fn verify_dbus_available() -> bool {
    // Check if dbus-send is available
    if let Ok(output) = Command::new("which")
        .arg("dbus-send")
        .output()
    {
        if output.status.success() {
            // Try to query GNOME SessionManager Presence interface (primary method)
            if let Ok(output) = Command::new("dbus-send")
                .args(&[
                    "--session",
                    "--print-reply",
                    "--dest=org.gnome.SessionManager",
                    "/org/gnome/SessionManager/Presence",
                    "org.freedesktop.DBus.Properties.Get",
                    "string:org.gnome.SessionManager.Presence",
                    "string:status",
                ])
                .output()
            {
                if output.status.success() {
                    return true;
                }
            }
            
            // Fallback: Try to query GNOME ScreenSaver interface (legacy)
            if let Ok(output) = Command::new("dbus-send")
                .args(&[
                    "--session",
                    "--print-reply",
                    "--dest=org.gnome.ScreenSaver",
                    "/org/gnome/ScreenSaver",
                    "org.gnome.ScreenSaver.GetActive",
                ])
                .output()
            {
                if output.status.success() {
                    return true;
                }
            }
            
            // Final fallback: Try to query session bus
            if let Ok(output) = Command::new("dbus-send")
                .args(&[
                    "--session",
                    "--print-reply",
                    "--dest=org.freedesktop.DBus",
                    "/org/freedesktop/DBus",
                    "org.freedesktop.DBus.ListNames",
                ])
                .output()
            {
                return output.status.success();
            }
        }
    }

    false
}

/// Polling-based lock detection fallback
/// Checks for screen lock status every 2 seconds
/// Uses multiple methods to detect screen lock state
fn monitor_polling(paths: &Paths) -> Result<()> {
    let mut was_locked = false;
    
    loop {
        let is_locked = check_screen_locked();
        
        // Detect transition from unlocked to locked
        if is_locked && !was_locked {
            eprintln!("🔒 System lock detected via polling");
            use crate::session::Session;
            let _ = Session::increment_lock_generation(paths);
            super::invalidate_all_sessions(paths)?;
            was_locked = true;
        }else if !is_locked && was_locked {
            eprintln!("🔓 System unlocked - sessions remain invalid until re-authentication");
            was_locked = false;
        }
        
        // Check every 2 seconds
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

/// Check if screen is currently locked using multiple detection methods
/// CRITICAL: Only return true for ACTUAL lock events, not idle/busy states
fn check_screen_locked() -> bool {
    // Method 1: Check GNOME SessionManager Presence status (primary method for GNOME)
    // Status 0 = available (unlocked), 1+ = locked/away/busy/idle
    // ONLY check for status 1 (invisible/locked)
    // DO NOT treat status 2 (busy) or status 3 (idle) as locked
    if let Ok(output) = Command::new("dbus-send")
        .args(&[
            "--session",
            "--print-reply",
            "--dest=org.gnome.SessionManager",
            "/org/gnome/SessionManager/Presence",
            "org.freedesktop.DBus.Properties.Get",
            "string:org.gnome.SessionManager.Presence",
            "string:status",
        ])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // ONLY status 1 means locked
            if output_str.contains("uint32 1") {
                return true;
            }
            // Status 0 = available (unlocked)
            // Status 2 = busy (NOT locked, user is active)
            // Status 3 = idle (NOT locked, just inactive)
            return false;
        }
    }
    
    // Method 2: Check GNOME ScreenSaver interface (legacy)
    if let Ok(output) = Command::new("dbus-send")
        .args(&[
            "--session",
            "--print-reply",
            "--dest=org.gnome.ScreenSaver",
            "/org/gnome/ScreenSaver",
            "org.gnome.ScreenSaver.GetActive",
        ])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("boolean true") {
                return true;
            }
        }
    }
    
    // Method 3: Check systemd login session lock status
    if let Ok(output) = Command::new("loginctl")
        .args(&["show-session", "-p", "LockedHint"])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("LockedHint=yes") {
                return true;
            }
        }
    }
    
    // // Method 4: Check if display is off (DPMS)
    // if let Ok(output) = Command::new("xset")
    //     .arg("q")
    //     .output()
    // {
    //     if output.status.success() {
    //         let output_str = String::from_utf8_lossy(&output.stdout);
    //         // Check if monitor is in standby/suspend/off mode
    //         if output_str.contains("Monitor is Off") || 
    //            output_str.contains("Monitor is Standby") ||
    //            output_str.contains("Monitor is Suspend") {
    //             return true;
    //         }
    //     }
    // }
    
    // Method 4: Check GNOME screensaver command (legacy)
    if let Ok(output) = Command::new("gnome-screensaver-command")
        .arg("--query")
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("is active") {
                return true;
            }
        }
    }
    
    // Method 5: Check for lock processes (i3lock, slock, etc.)
    if let Ok(output) = Command::new("pgrep")
        .arg("-x")
        .arg("i3lock")
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            return true;
        }
    }
    
    if let Ok(output) = Command::new("pgrep")
        .arg("-x")
        .arg("slock")
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            return true;
        }
    }
    
    // REMOVED Method: DPMS display off
    // Display being off does NOT mean system is locked
    // User might just have display set to turn off after inactivity
    
    false
}
