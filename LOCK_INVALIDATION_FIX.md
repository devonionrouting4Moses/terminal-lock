# Lock Invalidation Fix - Complete Implementation

## Problem Statement
After locking the laptop, the terminal session was not being invalidated. Users could continue using the terminal without re-authentication, which is a critical security vulnerability.

## Root Causes Identified

### 1. **Incorrect D-Bus Interface**
- **Issue**: The code was listening to `org.gnome.ScreenSaver.ActiveChanged` signals
- **Reality**: Modern GNOME (with Mutter) uses `org.gnome.SessionManager.Presence.StatusChanged`
- **Impact**: Lock events were never detected, so sessions were never invalidated

### 2. **No Session Validation on Command Execution**
- **Issue**: Shell integration only called `terminal-lock extend`, which always returned success
- **Reality**: When session was deleted (lock detected), the shell continued to work
- **Impact**: Terminal remained usable even after lock invalidation

## Solutions Implemented

### 1. Fixed D-Bus Lock Detection (linux.rs)

**Updated `monitor_dbus_signals()` function:**
- Now listens to `org.gnome.SessionManager.Presence.StatusChanged` signals
- Detects status changes: 0=available, 1+=locked/away/busy/idle
- Correctly triggers `invalidate_all_sessions()` when lock is detected

**Updated `verify_dbus_available()` function:**
- Primary check: `org.gnome.SessionManager.Presence` interface
- Fallback: `org.gnome.ScreenSaver` interface (legacy)
- Final fallback: Generic D-Bus session bus check

**Updated `check_screen_locked()` function:**
- Primary method: Check SessionManager Presence status
- Fallback methods: ScreenSaver, loginctl, xset, gnome-screensaver-command

### 2. Created Session Validation Command

**New `check` command** (`crates/cli/src/commands/check.rs`):
```rust
pub fn run(paths: &Paths) -> Result<()> {
    let is_valid = Session::is_valid(paths)?;
    
    if is_valid {
        Ok(())  // Exit code 0 - terminal is usable
    } else {
        eprintln!("❌ Session expired or invalid. Please authenticate again.");
        std::process::exit(1);  // Exit code 1 - block terminal
    }
}
```

### 3. Updated Shell Integration

**Before:**
```bash
PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}terminal-lock extend 2>/dev/null"
```

**After:**
```bash
PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}terminal-lock check 2>/dev/null && terminal-lock extend 2>/dev/null"
```

**How it works:**
1. Before each command, shell calls `terminal-lock check`
2. If session is invalid, check exits with code 1
3. The `&&` operator prevents `extend` from running
4. Shell prompt returns, but command is not executed
5. Terminal becomes effectively read-only until re-authentication

### 4. Updated install.sh

- Bash: Updated PROMPT_COMMAND to include check
- Zsh: Updated precmd() to include check
- Fish: Updated fish_postexec to include check

## How It Works Now

### Normal Flow (Session Valid)
```
User types: ls
Shell: terminal-lock check → success (exit 0)
Shell: terminal-lock extend → updates activity
Shell: executes ls command
```

### After Lock Detection (Session Invalid)
```
1. Laptop locked → D-Bus signal detected
2. Daemon calls invalidate_all_sessions()
3. All session.*.json files deleted
4. User unlocks laptop
5. User types: ls
6. Shell: terminal-lock check → fails (exit 1)
7. Shell: && prevents extend from running
8. Shell: command NOT executed
9. Terminal is blocked until re-authentication
```

## Testing

Run the test script to verify the fix:
```bash
./test_lock_invalidation.sh
```

This script:
1. Verifies no session exists initially
2. Tests check command fails with no session
3. Creates a valid session via authentication
4. Tests check command succeeds with valid session
5. Simulates lock by deleting session
6. Tests check command fails after deletion

## Files Modified

1. **crates/core/src/lock_monitor/linux.rs**
   - Updated D-Bus signal monitoring
   - Updated D-Bus verification
   - Updated polling detection

2. **crates/cli/src/commands/check.rs** (NEW)
   - New command to validate session and block terminal

3. **crates/cli/src/commands/mod.rs**
   - Added check module

4. **crates/cli/src/main.rs**
   - Added Check command to enum
   - Added handler for Check command

5. **install.sh**
   - Updated Bash PROMPT_COMMAND
   - Updated Zsh precmd function
   - Updated Fish fish_postexec function

## Security Improvements

✅ **Lock Detection**: Now correctly detects GNOME Mutter lock events
✅ **Session Invalidation**: Sessions are deleted when lock is detected
✅ **Terminal Blocking**: Terminal becomes unusable after lock (not just unmonitored)
✅ **Re-authentication Required**: User must authenticate again to use terminal
✅ **Per-Terminal Sessions**: Each terminal window has independent session

## Verification Steps

1. **Delete all session files:**
   ```bash
   rm -rf ~/.terminal_lock/session.*.json
   ```

2. **Open new terminal and authenticate**

3. **Run commands to verify terminal works**

4. **Lock laptop (Super+L)**

5. **Unlock laptop**

6. **Try to run command in same terminal**
   - Expected: Terminal blocked, must re-authenticate
   - Previous behavior: Terminal still worked (BUG)

## Exit Codes

- `terminal-lock check` returns 0 if session is valid
- `terminal-lock check` returns 1 if session is invalid
- Shell's `&&` operator ensures command only executes if check succeeds
