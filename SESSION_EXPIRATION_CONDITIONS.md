# Session Expiration Conditions - Complete Reference

## Overview
Sessions are automatically deleted/invalidated under the following conditions:

## Condition 1: Inactivity Timeout (5 Minutes)

### What Happens
- Session is deleted if no commands are executed for 5+ minutes
- Activity timestamp is updated after each command
- If user doesn't run any commands for 5 minutes, session becomes invalid

### Timeline
```
1. User authenticates → Session created with timestamp
2. User runs: ls
3. Activity timestamp updated
4. 5 minutes pass with NO commands
5. Session becomes invalid
6. Next prompt attempt → Validation fails
7. Terminal exits → User must re-authenticate
```

### Detection
- Checked in `Session::is_valid()` function
- Compares current time with `last_activity` timestamp
- If difference > 5 minutes → Session invalid

### Code Reference
```rust
// In crates/core/src/session.rs
pub fn is_valid(paths: &Paths) -> Result<bool> {
    // Check inactivity timeout (5 minutes)
    let now = Utc::now();
    let last_activity = session_info.last_activity;
    let inactivity_timeout = chrono::Duration::from_std(INACTIVITY_TIMEOUT).unwrap();
    
    if now.signed_duration_since(last_activity) > inactivity_timeout {
        return Ok(false); // Session invalid due to inactivity
    }
    
    Ok(true)
}
```

## Condition 2: Hard Timeout (15 Minutes)

### What Happens
- Session is deleted after 15 minutes regardless of activity
- Even if user is actively using terminal, session expires after 15 minutes
- Forces re-authentication every 15 minutes for security

### Timeline
```
1. User authenticates → Session created with expiry time
2. User runs commands continuously
3. 15 minutes pass
4. Session expires (hard timeout)
5. Next prompt attempt → Validation fails
6. Terminal exits → User must re-authenticate
```

### Detection
- Checked in `Session::is_valid()` function
- Compares current time with `expiry` timestamp
- If current time > expiry time → Session invalid

### Code Reference
```rust
// In crates/core/src/session.rs
pub fn is_valid(paths: &Paths) -> Result<bool> {
    // Check hard timeout (15 minutes)
    let now = Utc::now();
    let expiry = session_info.expiry;
    
    if now > expiry {
        return Ok(false); // Session expired (hard timeout)
    }
    
    Ok(true)
}
```

## Condition 3: System Lock Detection

### What Happens
- When laptop is locked (Super+L), all session files are immediately deleted
- D-Bus signal detected: `org.gnome.SessionManager.Presence.StatusChanged`
- Status changes from 0 (available) to 1/2/3 (locked/busy/idle)
- All sessions for all terminals are invalidated

### Timeline
```
1. User authenticates → Session created
2. User runs: ls
3. User locks laptop (Super+L)
4. Lock monitor detects lock signal
5. Prints: 🔒 System lock detected via D-Bus
6. Deletes ALL session files
7. User unlocks laptop
8. Lock monitor detects unlock signal
9. Prints: 🔓 System unlocked - sessions remain invalid
10. User tries to type: pwd
11. precmd() runs → Validates session
12. Session file doesn't exist → Validation fails
13. Terminal exits → User must re-authenticate
```

### Detection
- Monitored by `lock_monitor/linux.rs`
- Listens to D-Bus signals in real-time
- Immediately invalidates all sessions on lock

### Code Reference
```rust
// In crates/core/src/lock_monitor/linux.rs
if line.contains("uint32 1") || line.contains("uint32 2") || line.contains("uint32 3") {
    eprintln!("🔒 System lock detected via D-Bus (GNOME SessionManager Presence)");
    super::invalidate_all_sessions(paths)?;
} else if line.contains("uint32 0") {
    eprintln!("🔓 System unlocked - sessions remain invalid until re-authentication");
}
```

## Condition 4: Manual Session Cleanup

### What Happens
- When `source ~/.zshrc` is run, old session files are deleted
- New session is created for the refreshed shell
- Only one session file per terminal instance exists

### Timeline
```
1. User authenticates → Session created (session.706108.json)
2. User runs: source ~/.zshrc
3. Session cleanup triggered
4. Old session file deleted
5. New session created (session.714943.json)
6. User must re-authenticate
```

### Detection
- Implemented in `Session::create()` function
- Scans lock directory for old `session.*.json` files
- Deletes all old session files before creating new one

### Code Reference
```rust
// In crates/core/src/session.rs
pub fn create(paths: &Paths) -> Result<()> {
    // Clean up any old session files
    if let Ok(entries) = std::fs::read_dir(paths.lock_dir()) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.starts_with("session.") && filename.ends_with(".json") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
    // Create new session file...
}
```

## Summary Table

| Condition | Timeout | Trigger | Detection | Action |
|-----------|---------|---------|-----------|--------|
| Inactivity | 5 min | No commands | `is_valid()` check | Delete session |
| Hard Timeout | 15 min | Time elapsed | `is_valid()` check | Delete session |
| System Lock | Immediate | Lock detected | D-Bus signal | Delete all sessions |
| Manual Cleanup | N/A | `source ~/.zshrc` | Session creation | Delete old sessions |

## Shell Integration

### Zsh Configuration
```zsh
# Flag to track if this is the first prompt after auth
_TERMINAL_LOCK_FIRST_PROMPT=1

# Validate session before showing prompt (after command execution)
precmd() {
    # Skip validation on first prompt (right after authentication)
    if [ $_TERMINAL_LOCK_FIRST_PROMPT -eq 1 ]; then
        _TERMINAL_LOCK_FIRST_PROMPT=0
        return
    fi
    
    # Check session validity before showing next prompt
    terminal-lock check 2>/dev/null
    if [ $? -ne 0 ]; then
        # Session invalid - exit shell to force re-authentication
        echo ""
        echo "🔐 Session expired or invalid. Please authenticate again."
        exit 1
    fi
    
    # Update activity timestamp
    terminal-lock extend 2>/dev/null
}
```

### How It Works
1. **First prompt after auth**: Flag = 1 → Skip validation
2. **After first command**: Flag = 0 → Validate session
3. **Session valid**: Update activity, show next prompt
4. **Session invalid**: Exit shell, force re-authentication

## Testing All Conditions

### Test 1: Inactivity Timeout (5 minutes)
```bash
# Open terminal
# Authenticate with password
# Run: ls
# Wait 5+ minutes without typing
# Try to type: pwd
# Expected: Terminal exits, must re-authenticate
```

### Test 2: Hard Timeout (15 minutes)
```bash
# Open terminal
# Authenticate with password
# Run commands continuously for 15+ minutes
# Expected: After 15 minutes, terminal exits
```

### Test 3: System Lock Detection
```bash
# Open terminal
# Authenticate with password
# Run: ls
# Lock laptop (Super+L)
# Unlock laptop
# Try to type: pwd
# Expected: Terminal exits, must re-authenticate
```

### Test 4: Manual Session Cleanup
```bash
# Open terminal
# Authenticate with password
# Run: source ~/.zshrc
# Expected: Re-authenticate, only one session file
```

## Debug Output

### Lock Detection
```
🔵 Starting D-Bus lock monitor...
🔒 System lock detected via D-Bus (GNOME SessionManager Presence)
✓ Deleted session: "/home/user/.terminal_lock/session.706108.json"
```

### Unlock Detection
```
🔓 System unlocked - sessions remain invalid until re-authentication
```

### Session Validation Failure
```
🔐 Session expired or invalid. Please authenticate again.
```

### Session Creation
```
DEBUG: Creating session...
DEBUG: Lock dir: "/home/user/.terminal_lock"
DEBUG: Session file path: "/home/user/.terminal_lock/session.706108.json"
DEBUG: Session PID: 706108
DEBUG: Session file created successfully
```

## Security Guarantees

✅ **No illegal usage after inactivity** - Terminal blocked after 5 minutes
✅ **No illegal usage after hard timeout** - Terminal blocked after 15 minutes
✅ **No illegal usage after lock** - Terminal blocked immediately
✅ **Activity tracking** - Updated after each command
✅ **Session cleanup** - Old sessions deleted on refresh
✅ **Cross-terminal isolation** - Each terminal has independent session
✅ **Immediate lock response** - D-Bus signals processed in real-time

## Files Involved

1. **crates/core/src/session.rs**
   - `Session::is_valid()` - Validates session against timeouts
   - `Session::create()` - Creates new session, cleans old ones
   - `Session::update_last_activity()` - Updates activity timestamp

2. **crates/core/src/lock_monitor/linux.rs**
   - `monitor_dbus_signals()` - Listens for lock/unlock signals
   - `invalidate_all_sessions()` - Deletes all session files

3. **crates/cli/src/commands/check.rs**
   - `run()` - Validates session and exits if invalid

4. **crates/cli/src/commands/extend.rs**
   - `run()` - Updates activity timestamp

5. **~/.zshrc**
   - `precmd()` - Validates session before each prompt
   - `terminal-lock auth` - Authenticates on shell startup
   - `terminal-lock extend` - Updates activity after commands

## Configuration Constants

```rust
// In crates/core/src/session.rs
const SESSION_TTL: std::time::Duration = std::time::Duration::from_secs(15 * 60); // 15 minutes
const INACTIVITY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5 * 60); // 5 minutes
```

## Result

✅ **All four session expiration conditions implemented**
✅ **Inactivity timeout: 5 minutes**
✅ **Hard timeout: 15 minutes**
✅ **System lock detection: Immediate**
✅ **Manual cleanup: On shell refresh**
✅ **Terminal blocked when session invalid**
✅ **User must re-authenticate**
