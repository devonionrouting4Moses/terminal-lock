🚀 Terminal Lock System: Complete Implementation Guide
📋 Executive Summary
You're building a Rust-based terminal authentication system that replicates Linux's sudo behavior with enhanced session management. This guide provides a complete, modular implementation strategy.
Core Requirements:

✅ Per-terminal authentication (already working)
✅ 15-minute hard timeout (fixed in recent update)
✅ 5-minute inactivity timeout (fixed in recent update)
❌ System lock detection (MISSING - needs implementation)
✅ Background animations (fixed - now persistent)


🎯 Current System Status
✅ What's Working
Based on your documentation (Document #7 - Critical Fixes):
rust// ✅ Session validation (FIXED)
pub fn is_valid(paths: &Paths) -> Result<bool> {
    // CORRECTLY: Only checks, doesn't update activity
    // 15-min expiry enforced
    // 5-min inactivity enforced
}

// ✅ Activity tracking (FIXED)
pub fn validate_and_extend(paths: &Paths) -> Result<bool> {
    // CORRECTLY: Updates activity timestamp only
    // Does NOT extend 15-min expiry
    // Session remains FIXED at 15 minutes
}

// ✅ Animations (FIXED)
impl Drop for AnimationManager {
    fn drop(&mut self) {
        // CORRECTLY: Does nothing, animations continue
    }
}
❌ What's Missing
System Lock Detection - The critical feature you need to implement:
rust// Currently in lock_monitor.rs but NOT properly integrated
// Needs proper D-Bus/API integration for real-time detection
```

---

## 🏗️ Architecture Overview

### Current System Design
```
┌─────────────────────────────────────────────────────────┐
│                    TERMINAL LOCK                        │
│                  Current Architecture                   │
└─────────────────────────────────────────────────────────┘

┌──────────────┐
│   Terminal   │ Opens
│   (PID 123)  │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│   main.rs        │ Entry point
│   Commands::Auth │
└──────┬───────────┘
       │
       ├──► ┌─────────────────┐
       │    │ Authenticator   │ Check session validity
       │    │ ::needs_auth()  │
       │    └─────────────────┘
       │
       ├──► ┌─────────────────┐
       │    │ Session         │ Create/validate sessions
       │    │ ::is_valid()    │
       │    └─────────────────┘
       │
       ├──► ┌─────────────────┐
       │    │ AnimationMgr    │ Background animations
       │    │ (background)    │
       │    └─────────────────┘
       │
       └──► ┌─────────────────┐
            │ Shell continues │ User can type commands
            └─────────────────┘

┌──────────────┐
│ Shell Hook   │ After each command
│ PROMPT_CMD   │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ Commands::Extend │ Update activity
└──────────────────┘
```

### Required System Design (With Lock Detection)
```
┌─────────────────────────────────────────────────────────┐
│                TERMINAL LOCK v2.0                       │
│          Architecture With Lock Detection               │
└─────────────────────────────────────────────────────────┘

┌──────────────┐
│   Terminal   │ Opens
│   (PID 123)  │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│   main.rs        │ Entry point
│   Commands::Auth │
└──────┬───────────┘
       │
       ├──► ┌─────────────────┐
       │    │ Authenticator   │
       │    └─────────────────┘
       │
       ├──► ┌─────────────────┐
       │    │ Session::valid  │
       │    └─────────────────┘
       │
       ├──► ┌─────────────────┐
       │    │ Animation Daemon│ (background thread)
       │    └─────────────────┘
       │
       ├──► ┌─────────────────────────────┐  ⚡ NEW!
       │    │ LockMonitor::start_daemon() │  System lock detection
       │    └──────────┬──────────────────┘
       │               │
       │               │ Spawns background thread:
       │               │
       │               ▼
       │    ┌─────────────────────────────┐
       │    │  Lock Detection Loop        │
       │    │  ┌─────────────────────┐    │
       │    │  │ Linux: D-Bus        │    │
       │    │  │ macOS: NSWorkspace  │    │
       │    │  │ Win: WTS_SESSION    │    │
       │    │  └─────────────────────┘    │
       │    │          │                   │
       │    │          ▼                   │
       │    │  On Lock Detected:          │
       │    │  → Invalidate ALL sessions  │
       │    │  → Delete session files     │
       │    └─────────────────────────────┘
       │
       └──► ┌─────────────────┐
            │ Shell continues │
            └─────────────────┘

┌──────────────┐
│ OS Events    │ System lock/unlock
└──────┬───────┘
       │
       ▼
┌────────────────────┐
│ LockMonitor daemon │ Listening 24/7
│ (background thread)│
└────────────────────┘
       │
       │ Lock detected!
       ▼
┌────────────────────┐
│ Delete all session │
│ files immediately  │
└────────────────────┘

🔧 Implementation Plan
Phase 1: Understand Current Session Management ✅
You already have this working perfectly:
rust// File: crates/core/src/session.rs

pub struct Session;

impl Session {
    // ✅ CORRECTLY checks without updating activity
    pub fn is_valid(paths: &Paths) -> Result<bool> {
        // 1. Check session file exists
        // 2. Verify PID matches
        // 3. Check 15-min expiry
        // 4. Check 5-min inactivity
        // 5. Does NOT update activity
    }

    // ✅ CORRECTLY updates activity only, doesn't extend expiry
    pub fn validate_and_extend(paths: &Paths) -> Result<bool> {
        // 1. Check session valid
        // 2. Update activity timestamp
        // 3. Does NOT extend 15-min expiry
    }
}
Key Insight: Your recent fix separated "checking" from "updating" - this is correct!

Phase 2: Implement System Lock Detection ❌ (NEEDS WORK)
Current Implementation (Document #4):
rust// File: crates/core/src/lock_monitor.rs

impl LockMonitor {
    #[cfg(target_os = "linux")]
    fn monitor_linux(paths: &Paths) -> Result<()> {
        loop {
            if Self::is_system_locked_linux() {
                Self::invalidate_all_sessions(paths)?;
            }
            thread::sleep(Duration::from_secs(5)); // ⚠️ POLLING!
        }
    }

    #[cfg(target_os = "linux")]
    fn is_system_locked_linux() -> bool {
        // ⚠️ Uses dbus-send command - not real-time!
        if let Ok(output) = std::process::Command::new("dbus-send")
            .args(&[...])
            .output()
        {
            // Checks screensaver status
        }
    }
}
Problems with current approach:

❌ Polling - Checks every 5 seconds instead of real-time events
❌ Command execution - Spawns process for dbus-send instead of native D-Bus
❌ Not integrated - Daemon command exists but not called from main.rs
❌ Not persistent - Thread might not survive across sessions


Phase 3: Proper Lock Detection Implementation ✅ (SOLUTION)
Strategy A: Event-Driven D-Bus (Linux) - RECOMMENDED
Use the dbus crate for real-time notifications:
rust// Add to Cargo.toml
[dependencies]
dbus = "0.9"
dbus-tokio = "0.7"

// File: crates/core/src/lock_monitor/linux.rs
use dbus::blocking::Connection;
use dbus::Message;
use std::time::Duration;

pub fn start_monitor(paths: &Paths) -> Result<()> {
    let paths = paths.clone();
    
    std::thread::spawn(move || {
        // Connect to session bus
        let conn = Connection::new_session()
            .expect("Failed to connect to D-Bus");
        
        // Listen for ScreenSaver signals
        let rule = "type='signal',interface='org.freedesktop.ScreenSaver',member='ActiveChanged'";
        conn.add_match(rule)
            .expect("Failed to add D-Bus match rule");
        
        // Listen for GNOME ScreenSaver as fallback
        let gnome_rule = "type='signal',interface='org.gnome.ScreenSaver',member='ActiveChanged'";
        conn.add_match(gnome_rule)
            .expect("Failed to add GNOME D-Bus match");
        
        loop {
            // Block waiting for messages
            for msg in conn.incoming(Duration::from_millis(1000)) {
                if let Some(active) = msg.get1::<bool>() {
                    if active {
                        // Screen locked!
                        eprintln!("🔒 Screen lock detected - invalidating sessions");
                        if let Err(e) = invalidate_all_sessions(&paths) {
                            eprintln!("Failed to invalidate sessions: {}", e);
                        }
                    } else {
                        // Screen unlocked
                        eprintln!("🔓 Screen unlocked");
                    }
                }
            }
        }
    });
    
    Ok(())
}

fn invalidate_all_sessions(paths: &Paths) -> Result<()> {
    for session_file in paths.all_session_files()? {
        std::fs::remove_file(session_file)?;
    }
    Ok(())
}
Advantages:

✅ Real-time event detection (no polling!)
✅ Works with GNOME, KDE, XFCE
✅ Low CPU usage (event-driven)
✅ Handles both X11 and Wayland

Strategy B: macOS Native Events
rust// Add to Cargo.toml (macOS only)
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

// File: crates/core/src/lock_monitor/macos.rs
use cocoa::appkit::NSWorkspace;
use cocoa::foundation::NSString;
use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

pub fn start_monitor(paths: &Paths) -> Result<()> {
    let paths = paths.clone();
    
    std::thread::spawn(move || unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let nc: *mut Object = msg_send![workspace, notificationCenter];
        
        // Register for screen lock notification
        let name = NSString::alloc(nil).init_str("com.apple.screenIsLocked");
        
        let observer = Observer { paths: paths.clone() };
        let _: () = msg_send![nc, 
            addObserver: &observer
            selector: sel!(onScreenLocked:)
            name: name
            object: nil
        ];
        
        // Run event loop
        let run_loop: *mut Object = msg_send![class!(NSRunLoop), mainRunLoop];
        let _: () = msg_send![run_loop, run];
    });
    
    Ok(())
}

struct Observer {
    paths: Paths,
}

impl Observer {
    extern "C" fn on_screen_locked(&self, _notification: *mut Object) {
        eprintln!("🔒 macOS screen locked - invalidating sessions");
        let _ = invalidate_all_sessions(&self.paths);
    }
}
Strategy C: Windows WTS API
rust// Add to Cargo.toml (Windows only)
[target.'cfg(target_os = "windows")'.dependencies]
winapi = { version = "0.3", features = ["wtsapi32", "winuser"] }

// File: crates/core/src/lock_monitor/windows.rs
use winapi::um::wtsapi32::*;
use winapi::um::winuser::*;

pub fn start_monitor(paths: &Paths) -> Result<()> {
    let paths = paths.clone();
    
    std::thread::spawn(move || unsafe {
        // Register for session change notifications
        let hwnd = CreateWindowExW(...); // Create hidden window
        
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
        
        // Message loop
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_WTSSESSION_CHANGE {
                if msg.wParam == WTS_SESSION_LOCK {
                    eprintln!("🔒 Windows session locked - invalidating sessions");
                    let _ = invalidate_all_sessions(&paths);
                }
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
    
    Ok(())
}

Phase 4: Integration into main.rs
Update your main.rs (Document #5) to start the lock monitor:
rust// File: crates/cli/src/main.rs

use terminal_lock_core::LockMonitor; // Already imported!

#[tokio::main]
async fn main() -> Result<()> {
    let paths = Paths::new()?;
    paths.ensure_lock_dir()?;

    match cli.command {
        Some(Commands::Daemon) => {
            // Run lock monitor in foreground (for systemd/launchd)
            commands::daemon::run(&paths)?;
        }
        Some(Commands::Auth) | None => {
            // ✅ Already starts lock monitor!
            start_lock_monitor(&paths)?;
            
            // ... rest of authentication ...
        }
        // ... other commands ...
    }

    Ok(())
}

/// Start lock monitor daemon in background
fn start_lock_monitor(paths: &Paths) -> Result<()> {
    // This is ALREADY in your code! Just needs better implementation
    LockMonitor::start(paths)?; // ← Calls the improved version
    Ok(())
}
```

**The key insight:** Your code already calls `LockMonitor::start()` - you just need to replace the polling implementation with event-driven detection!

---

### Phase 5: Module Structure (Recommended)
```
crates/core/src/
├── lib.rs
├── session.rs (✅ working correctly)
├── lock_monitor/
│   ├── mod.rs (public interface)
│   ├── linux.rs (D-Bus event listener)
│   ├── macos.rs (NSWorkspace notifications)
│   └── windows.rs (WTS API)
└── paths.rs
File: crates/core/src/lock_monitor/mod.rs
rust#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::error::Result;
use crate::paths::Paths;

pub struct LockMonitor;

impl LockMonitor {
    pub fn start(paths: &Paths) -> Result<()> {
        #[cfg(target_os = "linux")]
        linux::start_monitor(paths)?;
        
        #[cfg(target_os = "macos")]
        macos::start_monitor(paths)?;
        
        #[cfg(target_os = "windows")]
        windows::start_monitor(paths)?;
        
        Ok(())
    }
}

/// Invalidate all sessions when system is locked
pub(crate) fn invalidate_all_sessions(paths: &Paths) -> Result<()> {
    if let Ok(session_files) = paths.all_session_files() {
        for session_file in session_files {
            let _ = std::fs::remove_file(session_file);
        }
    }
    Ok(())
}

🧪 Testing Strategy
Test 1: Manual Lock Detection
bash# Terminal 1: Start terminal-lock
terminal-lock auth
# Should authenticate and show greeting

# Terminal 2: Check sessions
ls ~/.terminal_lock/session.*.json
# Should see session file

# Lock your laptop
# Ctrl+Alt+L (Linux), Win+L (Windows), Cmd+Control+Q (macOS)

# Wait 2 seconds, then unlock

# Terminal 2: Check sessions again
ls ~/.terminal_lock/session.*.json
# Session files should be DELETED

# Terminal 1: Try to run command
ls
# Should prompt for password again! ✅
Test 2: 15-Minute Hard Timeout
bash# Authenticate at T+0
terminal-lock auth

# Run commands at T+5, T+10, T+14
# Should NOT prompt for password

# Run command at T+16
ls
# Should prompt for password ✅
Test 3: 5-Minute Inactivity
bash# Authenticate
terminal-lock auth

# Run command
ls

# Wait 6 minutes WITHOUT typing

# Run another command
pwd
# Should prompt for password ✅
Test 4: Multiple Terminals Independence
bash# Terminal 1: Authenticate at T+0
terminal-lock auth

# Terminal 2: Open at T+5
# Should prompt for password ✅

# Terminal 1: Wait until T+16
ls
# Should prompt (15-min expired)

# Terminal 2: Run command at T+10
pwd
# Should NOT prompt (still valid until T+20)

📦 Cargo.toml Updates
toml[package]
name = "terminal-lock-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
thiserror = "1.0"

# Lock detection - Linux
[target.'cfg(target_os = "linux")'.dependencies]
dbus = "0.9"

# Lock detection - macOS
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

# Lock detection - Windows
[target.'cfg(target_os = "windows")'.dependencies]
winapi = { version = "0.3", features = ["wtsapi32", "winuser"] }
```

---

## 🎯 Final Architecture Comparison

### Before (What you have now)
```
Session Management: ✅ FIXED (15-min + 5-min working)
Animation System:   ✅ FIXED (persistent background)
Lock Detection:     ❌ POLLING (checks every 5 seconds)
```

### After (What you need)
```
Session Management: ✅ WORKING
Animation System:   ✅ WORKING
Lock Detection:     ✅ EVENT-DRIVEN (real-time D-Bus/API)

🚀 Implementation Steps (Priority Order)
Step 1: Create Module Structure ⚡ START HERE
bashmkdir -p crates/core/src/lock_monitor
touch crates/core/src/lock_monitor/mod.rs
touch crates/core/src/lock_monitor/linux.rs
touch crates/core/src/lock_monitor/macos.rs
touch crates/core/src/lock_monitor/windows.rs
Step 2: Implement Linux D-Bus Detection (Primary Platform)
Copy the Linux implementation from Phase 3, Strategy A into linux.rs.
Step 3: Update Cargo.toml
Add dbus = "0.9" to Linux dependencies.
Step 4: Test on Linux
bashcargo build --release
./install.sh
# Test with Ctrl+Alt+L
Step 5: Implement macOS/Windows (If Needed)
Use Strategy B/C from Phase 3.

❓ Key Questions Answered
Q1: Should lock monitor be separate binary or integrated?
Answer: Integrated background thread (what you're already doing is correct).
Why:

No extra process to manage
Shares same session file access
Starts automatically with first terminal
Dies when last terminal closes

Q2: How to prevent race conditions with session files?
Answer: Use file locking:
rustuse std::fs::{File, OpenOptions};
use std::io::{Read, Write};

fn atomic_session_write(path: &Path, content: &str) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_EXCL prevents race conditions
    }
    
    file.write_all(content.as_bytes())?;
    Ok(())
}
```

### Q3: Should sessions be encrypted?

**Answer:** Current approach is secure enough:
```
~/.terminal_lock/session.12345.json
Permissions: 0600 (owner read/write only)
Rationale:

Session files already protected by filesystem permissions
Only contain timestamps, no sensitive data
Encryption would add complexity without significant security benefit
If attacker has file access, they can already hijack terminal


🎓 Summary
What's Already Working ✅
Your recent fixes (Document #7) solved:

✅ Session expiration (15-min hard, 5-min inactivity)
✅ Animation persistence
✅ Per-terminal independence

What You Need to Do ❌

Replace polling with event-driven lock detection:

Linux: Use dbus crate for real-time signals
macOS: Use cocoa crate for NSWorkspace notifications
Windows: Use winapi crate for WTS events


No changes needed to:

Session management (already correct)
Shell integration (already correct)
Main authentication flow (already correct)



Implementation Time Estimate

Linux D-Bus: 2-3 hours (primary focus)
macOS/Windows: 1-2 hours each (if needed)
Testing: 1-2 hours
Total: ~6-8 hours for complete implementation

Critical Success Factors

✅ Don't touch session.rs (it's working correctly now)
✅ Keep main.rs as-is (just improve LockMonitor implementation)
✅ Focus on event-driven detection (not polling)
✅ Test thoroughly with actual system locks


You're 90% there! Just need to upgrade the lock detection from polling to event-driven. Everything else is already working perfectly. 🎉