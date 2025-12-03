# Terminal Lock - Quick Reference Guide

## What Was Implemented

Following `docs/impl/auth.md` line by line, the Terminal Lock system now includes:

### ✅ Core Features (Already Working)
- **15-minute hard timeout**: Sessions expire exactly 15 minutes after creation
- **5-minute inactivity timeout**: Sessions expire if idle for 5+ minutes
- **Per-terminal independence**: Each terminal has its own session (PID-based)
- **Shell integration**: Works with Bash, Zsh, and Fish
- **Non-blocking main process**: Returns control to shell immediately

### ✅ NEW: System Lock Detection
- **Linux**: D-Bus based detection (GNOME, KDE, XFCE compatible)
- **macOS**: AppleScript based detection
- **Windows**: WMI based detection
- **Automatic invalidation**: All sessions deleted when system locks
- **Background monitoring**: Runs in separate thread, checks every 5 seconds

---

## Key Files

| File | Purpose |
|------|---------|
| `crates/core/src/lock_detection.rs` | System lock detection (NEW) |
| `crates/core/src/session.rs` | Session management (15-min + 5-min timeouts) |
| `crates/cli/src/commands/daemon.rs` | Daemon command (NEW) |
| `crates/cli/src/main.rs` | Main entry point (lock monitor integration) |
| `install.sh` | Shell integration (Bash, Zsh, Fish) |

---

## How It Works

### When Terminal Opens
```
1. Start lock monitor daemon (background thread)
2. Check if session exists and is valid
   - Check 15-minute hard timeout
   - Check 5-minute inactivity timeout
   - Verify PID matches
3. If invalid: Prompt for password
4. If valid: Show greeting
5. Return control to shell
```

### After Each Command
```
1. PROMPT_COMMAND/precmd hook runs
2. terminal-lock extend is called
3. Activity timestamp is updated
4. Session validity is checked
5. If expired: User prompted for password on next command
```

### When System Locks
```
1. Lock monitor detects lock event
2. All session files are deleted
3. Next terminal interaction requires re-authentication
```

---

## Commands

```bash
# Authenticate this terminal
terminal-lock auth

# Extend session (called automatically by shell)
terminal-lock extend

# Start lock monitor daemon
terminal-lock daemon

# Check session status
terminal-lock status

# Lock terminal manually
terminal-lock lock

# Reset attempts counter
terminal-lock reset

# Configure preferences
terminal-lock preferences

# Initial setup
terminal-lock setup
```

---

## Testing

### Test 1: Lock Detection
```bash
# 1. Open terminal and authenticate
terminal-lock auth

# 2. Lock system (Ctrl+Alt+L on Linux)

# 3. Unlock system

# 4. Type command in terminal
# Expected: Prompted for password
```

### Test 2: Inactivity (5 minutes)
```bash
# 1. Authenticate
terminal-lock auth

# 2. Run command
ls

# 3. Wait 5+ minutes without typing

# 4. Type command
ls

# Expected: Prompted for password
```

### Test 3: Hard Timeout (15 minutes)
```bash
# 1. Authenticate at 10:00
terminal-lock auth

# 2. Run commands every 2 minutes (keeps activity updated)
# 10:02, 10:04, 10:06, 10:08, 10:10, 10:12, 10:14

# 3. At 10:16, type command
ls

# Expected: Prompted for password (15-min hard timeout)
```

### Test 4: Multiple Terminals
```bash
# Terminal 1: Authenticate
terminal-lock auth

# Terminal 2: Open new terminal
# Expected: Prompted for password (independent session)

# Each terminal has separate 15-minute timeout
```

---

## Build & Install

```bash
# Build release binary
cargo build --release

# Run installer
./install.sh

# Verify installation
terminal-lock status
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│         Terminal Opens                  │
└────────────────┬────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │ Start Lock Monitor         │
    │ (Background Thread)        │
    │ - Monitors system lock     │
    │ - Invalidates sessions     │
    └────────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │ Check Session Valid?       │
    │ - 15-min hard timeout      │
    │ - 5-min inactivity         │
    │ - PID verification         │
    └────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
        ▼                 ▼
    ┌────────┐      ┌──────────────┐
    │ Valid  │      │ Invalid      │
    └────┬───┘      └────┬─────────┘
         │               │
         ▼               ▼
    ┌────────┐      ┌──────────────┐
    │ Show   │      │ Prompt for   │
    │Greeting│      │ Password     │
    └────┬───┘      └────┬─────────┘
         │               │
         └───────┬───────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │ Return to Shell            │
    │ (Main process exits)       │
    └────────────────────────────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │ Shell Prompt Ready         │
    │ User can type commands     │
    └────────────────────────────┘
```

---

## Implementation Details

### Session File
- **Location**: `~/.terminal_lock/session.<PID>.json`
- **Format**: JSON with expiry and terminal_id
- **Permissions**: 0o600 (user only)

### Activity File
- **Location**: `~/.terminal_lock/activity.<PID>`
- **Content**: Last activity timestamp (RFC3339)
- **Permissions**: 0o600 (user only)

### Lock Detection
- **Linux**: D-Bus via `dbus-send` command
- **macOS**: AppleScript via `osascript` command
- **Windows**: WMI via `wmic` command
- **Polling**: Every 5 seconds
- **Thread**: Background, non-blocking

### Session Timeouts
- **Hard Timeout**: 15 minutes from creation (never extends)
- **Inactivity Timeout**: 5 minutes of no activity
- **Activity Update**: After each command (via PROMPT_COMMAND/precmd)
- **Check Frequency**: Every shell prompt

---

## Troubleshooting

### Sessions not expiring
- Check that `PROMPT_COMMAND` or `precmd` is set correctly
- Verify `terminal-lock extend` is being called
- Check session file exists in `~/.terminal_lock/`

### Lock detection not working
- Verify `dbus-send` is installed (Linux)
- Verify `osascript` is available (macOS)
- Verify `wmic` is available (Windows)
- Check that lock monitor daemon is running

### Terminal not prompting for password
- Check session file hasn't been manually deleted
- Verify PID matches in session file
- Check that 15 minutes haven't passed since authentication

---

## Security Notes

- Session files are user-readable/writable only (0o600)
- Activity files are user-readable/writable only (0o600)
- No elevated privileges required
- Lock detection uses platform-native APIs
- Graceful fallbacks for compatibility

---

## Requirements Verification

All requirements from `docs/impl/auth.md` have been implemented:

- ✅ System lock detection (Linux, macOS, Windows)
- ✅ 15-minute hard timeout
- ✅ 5-minute inactivity timeout
- ✅ Per-terminal independence
- ✅ Shell integration (Bash, Zsh, Fish)
- ✅ Non-blocking main process
- ✅ Background daemon
- ✅ Cross-platform support
- ✅ No infinite loops
- ✅ No blocking on lock detection

**Status**: ✅ **READY FOR DEPLOYMENT**

See `REQUIREMENTS_IMPLEMENTATION_MAP.md` for detailed mapping of requirements to implementation.
