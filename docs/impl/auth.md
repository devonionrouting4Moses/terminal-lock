Comprehensive Implementation Prompt for Terminal Lock Authentication System
Context & Goal
I'm building a Rust-based terminal authentication system called "Terminal Lock" that mimics Linux's sudo behavior. The system should:

Prompt users for authentication when opening new terminal sessions
Automatically expire sessions after 15 minutes (hard timeout)
Automatically expire sessions after 5 minutes of inactivity
Lock terminals when the laptop/system locks
Maintain independent sessions per terminal (each terminal PID has its own session)

Current Architecture Overview
Project Structure
terminal-lock/
├── crates/
│   ├── cli/          # Command-line interface
│   │   ├── main.rs   # Entry point, command routing
│   │   └── commands/ # Individual command implementations
│   └── core/         # Core business logic
│       └── session.rs # Session management
└── install.sh        # Shell integration installer
Key Components

Session Management (crates/core/src/session.rs): Handles session creation, validation, and expiration
Authentication Flow (crates/cli/src/main.rs): Entry point that checks authentication status
Shell Integration (install.sh): Hooks into bash/zsh/fish to call authentication checks

Problems I've Encountered
Based on the provided documentation, these issues were previously fixed but may regress:

Infinite loop blocking shell - Main process hangs instead of returning control to shell
Sessions never expiring - 15-minute timeout not enforced
Inactivity timeout not triggering - 5-minute idle timeout not working
Shell becoming unresponsive - Terminal freezes after authentication

Required Functionality
1. System Lock Detection
MISSING FEATURE - Need to Implement:
When the user locks their laptop (Ctrl+Alt+L on Linux, Win+L on Windows, Cmd+Control+Q on macOS):

All active terminal sessions should be immediately invalidated
Session files should be deleted or marked as expired
Next terminal interaction should require re-authentication

Implementation Requirements:

Listen for system lock/unlock events (D-Bus on Linux, Windows API on Windows, macOS framework on macOS)
Run a background daemon/service that monitors lock status
Immediately invalidate all session files when lock is detected
Must work across different Linux desktop environments (GNOME, KDE, XFCE, etc.)

2. Session Expiration Mechanisms
Hard Timeout (15 minutes)

Session expires exactly 15 minutes after creation, regardless of activity
Implemented via expiry timestamp in session file
Must be checked on every shell prompt

Inactivity Timeout (5 minutes)

Session expires after 5 minutes of no terminal activity
Implemented via last_activity timestamp in session file
Updated after each command execution
Must be checked on every shell prompt

3. Per-Terminal Independence

Each terminal window/tab has its own session (identified by PID)
Opening a new terminal requires fresh authentication
Sessions don't share state between terminals
Session file naming: session.<PID>.json

4. Shell Integration
Must work seamlessly with:

Bash (via PROMPT_COMMAND)
Zsh (via precmd hook)
Fish (via fish_postexec event)

Technical Implementation Guidelines
Session File Structure
json{
  "expiry": "2025-11-22T10:15:00Z",      // 15 min from creation
  "last_activity": "2025-11-22T10:00:00Z", // Updated on each command
  "terminal_id": "12345"                  // Process PID
}
Critical Code Flow
1. Terminal Startup (main.rs)
rust// Pseudocode - DO NOT use infinite loops!
fn main() {
    let terminal_pid = get_current_pid();
    
    // Check if session exists and is valid
    if !session_valid_for_pid(terminal_pid) {
        // Prompt for password
        authenticate_user();
        // Create session with current PID
        create_session(terminal_pid);
    }
    
    // Show greeting
    show_greeting();
    
    // CRITICAL: Return control to shell - DO NOT BLOCK!
    return Ok(());
}
2. Shell Integration Hook
bash# Called after every command
PROMPT_COMMAND="terminal-lock extend 2>/dev/null"
3. Extend Command (commands/extend.rs)
rust// Pseudocode
fn extend_session() {
    let terminal_pid = get_current_pid();
    let session = load_session(terminal_pid);
    
    // Check if session is still valid
    if session.is_expired() || session.is_inactive() {
        // Session expired - require re-auth
        delete_session(terminal_pid);
        return Err("Session expired");
    }
    
    // Update last_activity timestamp
    session.update_activity();
    
    // Extend expiry (reset 15-min timer)
    session.extend_expiry();
    
    // Save updated session
    save_session(terminal_pid, session);
}
System Lock Detection (NEW - Must Implement)
Linux (D-Bus)
rust// Pseudocode for Linux lock detection
fn start_lock_monitor_linux() {
    // Connect to D-Bus session bus
    // Listen for org.freedesktop.ScreenSaver signals
    // OR org.gnome.ScreenSaver for GNOME
    // OR org.freedesktop.login1.Session for systemd
    
    on_lock_signal(|signal| {
        if signal == "Lock" {
            invalidate_all_sessions();
        }
    });
}
macOS (Distributed Notification Center)
rust// Pseudocode for macOS lock detection
fn start_lock_monitor_macos() {
    // Listen for com.apple.screenIsLocked notification
    // Listen for com.apple.screenIsUnlocked notification
    
    on_lock_notification(|notification| {
        if notification == "screenIsLocked" {
            invalidate_all_sessions();
        }
    });
}
Windows (WTS API)
rust// Pseudocode for Windows lock detection
fn start_lock_monitor_windows() {
    // Register for WTS_SESSION_LOCK event
    // Use WTSRegisterSessionNotification
    
    on_wts_event(|event| {
        if event == WTS_SESSION_LOCK {
            invalidate_all_sessions();
        }
    });
}
```

## Common Pitfalls to Avoid

### ❌ Don't Do This:
1. **Infinite loops in main process** - Blocks shell from functioning
2. **Single session file for all terminals** - Breaks per-terminal independence
3. **Checking session only at startup** - Timeouts never trigger
4. **Forgetting to update last_activity** - Inactivity timeout never works
5. **Blocking on lock detection** - Should be async/background daemon
6. **Platform-specific hardcoding** - Must detect and adapt to OS

### ✅ Do This:
1. **Return control to shell immediately** after authentication
2. **Use PID-based session files** (`session.<PID>.json`)
3. **Check session validity on every prompt** (via shell hooks)
4. **Update activity timestamp after every command**
5. **Run lock monitor as separate background service**
6. **Use conditional compilation** for platform-specific code

## Implementation Checklist

### Phase 1: Fix Core Session Management
- [ ] Remove any infinite loops from main.rs
- [ ] Ensure main() returns after authentication
- [ ] Implement `Session::validate_and_extend()` method
- [ ] Register `extend` command in CLI
- [ ] Add shell integration hooks (PROMPT_COMMAND, precmd, postexec)

### Phase 2: Add System Lock Detection
- [ ] Create background daemon/service
- [ ] Implement Linux lock detection (D-Bus)
- [ ] Implement macOS lock detection (Distributed Notifications)
- [ ] Implement Windows lock detection (WTS API)
- [ ] Test lock detection on all platforms
- [ ] Ensure daemon starts automatically with system

### Phase 3: Testing & Validation
- [ ] Test 15-minute hard timeout
- [ ] Test 5-minute inactivity timeout
- [ ] Test system lock invalidation
- [ ] Test multiple terminals with independent sessions
- [ ] Test cross-platform compatibility
- [ ] Test shell integration on bash/zsh/fish

## Platform-Specific Requirements

### Linux
- Dependencies: `dbus`, `systemd` (optional)
- Desktop environments to support: GNOME, KDE, XFCE, i3, others
- Must handle both X11 and Wayland

### macOS
- Dependencies: `CoreFoundation`, `Foundation`
- Must request necessary permissions for system events

### Windows
- Dependencies: `winapi`, `wtsapi32`
- Must handle both console and terminal emulators

## Example Test Scenarios

### Scenario 1: System Lock
```
1. Open terminal, authenticate successfully
2. Run some commands (session extends)
3. Lock laptop with Ctrl+Alt+L (Linux) or Win+L (Windows)
4. Unlock laptop
5. Return to terminal, type command
6. Expected: Prompted for password (session invalidated by lock)
```

### Scenario 2: Inactivity Timeout
```
1. Open terminal, authenticate
2. Run command at 10:00
3. Wait until 10:06 without typing anything
4. Type new command
5. Expected: Prompted for password (5-min inactivity exceeded)
```

### Scenario 3: Hard Timeout
```
1. Authenticate at 10:00
2. Run commands every 2 minutes (10:02, 10:04, 10:06, 10:08, 10:10, 10:12, 10:14)
3. At 10:16, type command
4. Expected: Prompted for password (15-min hard timeout exceeded)
Questions for AI Assistant
When implementing this, please address:

Lock Detection Architecture: Should the lock monitor be a separate binary/service, or integrated into the main CLI? How should it communicate session invalidation?
Cross-Platform Strategy: What's the best approach for conditional compilation and platform detection? Should I use feature flags?
Session File Locking: How do I prevent race conditions when multiple terminals try to extend/validate sessions simultaneously?
Daemon Management: How should the lock monitor daemon be started and managed? Should it be a systemd service on Linux, LaunchDaemon on macOS, Windows Service on Windows?
Error Handling: What should happen if lock detection fails or is unavailable on a platform?
Security Considerations: Are there any security risks with the current session file approach? Should sessions be encrypted?

Desired Output
Please provide:

Complete implementation of system lock detection for all platforms
Fixed authentication flow that doesn't block the shell
Proper session validation and extension logic
Shell integration that works reliably
Step-by-step instructions for testing each feature
Recommendations for improving the architecture if needed

Additional Context

Language: Rust (latest stable)
Target platforms: Linux, macOS, Windows
Must work in: Terminal emulators, VS Code integrated terminal, native OS terminals
Should NOT require root/admin privileges to run
Existing code already has basic password hashing and storage working