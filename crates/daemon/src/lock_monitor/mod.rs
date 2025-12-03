//! Daemon lock monitor - Detects system lock/unlock events
//! Pauses all sessions on lock, resumes on unlock

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::daemon::Daemon;
use terminal_lock_ipc::DaemonRequest;

/// Start lock monitor for daemon
/// Detects system lock/unlock events and pauses/resumes sessions
pub fn start(daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    thread::spawn(move || {
        #[cfg(target_os = "linux")]
        {
            eprintln!("🔵 Daemon: Starting Linux D-Bus lock monitor...");
            if let Err(e) = linux_monitor(daemon) {
                eprintln!("🔴 Daemon lock monitor error: {}", e);
            }
        }

        #[cfg(target_os = "macos")]
        {
            eprintln!("🔵 Daemon: Starting macOS lock monitor...");
            if let Err(e) = macos_monitor(daemon) {
                eprintln!("🔴 Daemon lock monitor error: {}", e);
            }
        }

        #[cfg(target_os = "windows")]
        {
            eprintln!("🔵 Daemon: Starting Windows lock monitor...");
            if let Err(e) = windows_monitor(daemon) {
                eprintln!("🔴 Daemon lock monitor error: {}", e);
            }
        }
    });

    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_monitor(daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::{Command, Stdio};
    use std::io::{BufRead, BufReader};

    loop {
        // Monitor session bus for GNOME SessionManager Presence signals
        // org.gnome.SessionManager runs on the session bus, not system bus
        eprintln!("🔵 Connecting to D-Bus session bus for lock monitoring...");
        if let Ok(child) = Command::new("dbus-monitor")
            .args(&[
                "--session",
                "type='signal',interface='org.gnome.SessionManager.Presence',member='StatusChanged'",
            ])
            .stdout(Stdio::piped())
            .spawn()
        {
            if let Some(stdout) = child.stdout {
                let reader = BufReader::new(stdout);
                eprintln!("✅ Connected to session bus - listening for lock events");
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.contains("StatusChanged") {
                            if line.contains("uint32 1") {
                                // Lock detected (status 1 = invisible/locked)
                                eprintln!("🔒 System locked - pausing all sessions");
                                let store = daemon.session_store.lock().unwrap();
                                let _ = store.pause_all_sessions();
                                let _ = daemon.state.lock().unwrap().lock_generation += 1;
                            } else if line.contains("uint32 0") {
                                // Unlock detected (status 0 = available)
                                eprintln!("🔓 System unlocked - resuming sessions");
                                let store = daemon.session_store.lock().unwrap();
                                let _ = store.resume_all_sessions();
                            }
                        }
                    }
                }
            }
        } else {
            // Fallback to --session bus
            // eprintln!("⚠️  --system bus failed, falling back to --session bus");
            // if let Ok(child) = Command::new("dbus-monitor")
            //     .args(&[
            //         "--session",
            //         "type='signal',interface='org.gnome.SessionManager.Presence',member='StatusChanged'",
            //     ])
            //     .stdout(Stdio::piped())
            //     .spawn()
            // {
            //     if let Some(stdout) = child.stdout {
            //         let reader = BufReader::new(stdout);
            //         eprintln!("✅ Connected to --session bus");
            //         for line in reader.lines() {
            //             if let Ok(line) = line {
            //                 if line.contains("StatusChanged") {
            //                     if line.contains("uint32 1") {
            //                         // Lock detected
            //                         eprintln!("🔒 System locked - pausing all sessions");
            //                         let store = daemon.session_store.lock().unwrap();
            //                         let _ = store.pause_all_sessions();
            //                         let _ = daemon.state.lock().unwrap().lock_generation += 1;
            //                     } else if line.contains("uint32 0") {
            //                         // Unlock detected
            //                         eprintln!("🔓 System unlocked - resuming sessions");
            //                         let store = daemon.session_store.lock().unwrap();
            //                         let _ = store.resume_all_sessions();
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }
            eprintln!("⚠️  Failed to connect to D-Bus session bus");
        }

        // Reconnect if monitor dies
        eprintln!("📍 D-Bus monitor disconnected, reconnecting in 5 seconds...");
        thread::sleep(Duration::from_secs(5));
    }
}

#[cfg(target_os = "macos")]
fn macos_monitor(daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // macOS screensaver detection via polling
        thread::sleep(Duration::from_secs(2));
        
        // Check if screensaver is running (indicates lock)
        let output = std::process::Command::new("defaults")
            .args(&["read", "com.apple.screensaver", "idleTime"])
            .output()?;

        if output.status.success() {
            // If we can read it, screensaver might be active
            let store = daemon.session_store.lock().unwrap();
            let _ = store.pause_all_sessions();
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_monitor(daemon: Arc<Daemon>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Windows workstation lock detection via polling
        thread::sleep(Duration::from_secs(2));
        
        // Check if workstation is locked
        let output = std::process::Command::new("powershell")
            .args(&["-Command", "[System.Windows.Forms.SystemInformation]::UserIdle"])
            .output()?;

        if output.status.success() {
            let store = daemon.session_store.lock().unwrap();
            let _ = store.pause_all_sessions();
        }
    }
}
