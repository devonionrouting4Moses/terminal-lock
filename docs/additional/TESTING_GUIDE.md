# Terminal Lock - Testing Guide

## Quick Summary of Fixes

| Issue | Root Cause | Fix |
|-------|-----------|-----|
| 15-min expiration not working | Infinite loop blocked shell re-checks | Removed loop, let shell continue |
| 5-min inactivity not detected | Check only ran once at startup | Added shell integration to call extend after each command |
| Missing method | `validate_and_extend()` didn't exist | Implemented method in session.rs |

---

## Build & Install

```bash
cd terminal-lock

# Build the project
cargo build --release

# Install (or reinstall)
./install.sh
```

---

## Test 1: 15-Minute Session Expiration

**Objective:** Verify session expires exactly 15 minutes after creation

**Steps:**

1. **Open terminal and note the time**
   ```bash
   date  # Note: 8:45 AM
   ```

2. **Authenticate**
   ```bash
   # Terminal will prompt for password
   # Enter your password
   ```

3. **Verify session was created**
   ```bash
   terminal-lock status
   # Should show: Session valid until 9:00 AM (15 minutes from now)
   ```

4. **Wait 15 minutes** (or modify system clock for testing)

5. **Type a command**
   ```bash
   ls
   ```

6. **Expected Result:**
   - ✅ Prompted for password again
   - ✅ Session expired message shown
   - ✅ New session created after authentication

**Verification:**
```bash
# Check session file was recreated
ls -la ~/.terminal_lock/session.*.json
# Should show recent timestamp
```

---

## Test 2: 5-Minute Inactivity Timeout

**Objective:** Verify session expires after 5 minutes of inactivity

**Steps:**

1. **Open terminal and authenticate**
   ```bash
   # Terminal prompts for password
   # Enter password
   ```

2. **Verify session created**
   ```bash
   terminal-lock status
   # Should show: Session valid, no inactivity
   ```

3. **Run a command**
   ```bash
   ls
   ```

4. **Check activity timestamp**
   ```bash
   cat ~/.terminal_lock/activity.*.txt
   # Should show current time
   ```

5. **Wait 5+ minutes without typing anything**
   - Do NOT run any commands
   - Do NOT press Enter
   - Just wait

6. **Type a command after 5+ minutes**
   ```bash
   pwd
   ```

7. **Expected Result:**
   - ✅ Prompted for password
   - ✅ Inactivity timeout message shown
   - ✅ New session created after authentication

**Verification:**
```bash
# Check that inactivity was detected
# The session file should have been deleted
ls ~/.terminal_lock/session.*.json
# Should NOT exist or should be newly created
```

---

## Test 3: Session Extension on Activity

**Objective:** Verify session extends when user is active

**Steps:**

1. **Open terminal and authenticate**
   ```bash
   # Enter password
   ```

2. **Note the expiry time**
   ```bash
   terminal-lock status
   # Note: expires at 9:00 AM
   ```

3. **Run commands every 2 minutes for 20 minutes**
   ```bash
   # At 8:50
   ls
   terminal-lock status  # Should show: expires at 9:05
   
   # At 8:52
   pwd
   terminal-lock status  # Should show: expires at 9:07
   
   # At 8:54
   date
   terminal-lock status  # Should show: expires at 9:09
   
   # ... continue for 20 minutes
   ```

4. **Expected Result:**
   - ✅ Session never expires
   - ✅ Expiry time keeps extending by 15 minutes
   - ✅ Never prompted for password
   - ✅ Activity timestamp updates after each command

**Verification:**
```bash
# Check activity file timestamp
cat ~/.terminal_lock/activity.*.txt
# Should show recent time (within last 2 minutes)
```

---

## Test 4: Multiple Independent Terminals

**Objective:** Verify each terminal has independent session

**Steps:**

1. **Terminal 1: Open and authenticate at 8:45**
   ```bash
   # Terminal 1 opens
   # Prompted for password
   # Enter password
   # Session created: expires at 9:00
   ```

2. **Terminal 2: Open and authenticate at 8:50**
   ```bash
   # Terminal 2 opens
   # Prompted for password (even though Terminal 1 is open!)
   # Enter password
   # Session created: expires at 9:05
   ```

3. **Terminal 1: Wait until 9:00**
   ```bash
   # At 9:00, type a command
   ls
   # Expected: Prompted for password
   ```

4. **Terminal 2: Still valid until 9:05**
   ```bash
   # At 9:00, type a command
   pwd
   # Expected: Works! (no password prompt)
   ```

5. **Terminal 2: Wait until 9:05**
   ```bash
   # At 9:05, type a command
   date
   # Expected: Prompted for password
   ```

**Expected Result:**
- ✅ Each terminal requires independent authentication
- ✅ Each terminal has its own 15-minute session
- ✅ Sessions expire independently
- ✅ No session sharing between terminals

**Verification:**
```bash
# In each terminal, check session files
ls ~/.terminal_lock/session.*.json
# Terminal 1 should have: session.12345.json (its PID)
# Terminal 2 should have: session.12346.json (its PID)
```

---

## Test 5: Shell Integration (PROMPT_COMMAND / precmd)

**Objective:** Verify shell integration calls extend after each command

**Steps:**

1. **Enable debug logging**
   ```bash
   export RUST_LOG=debug
   ```

2. **Open terminal and authenticate**
   ```bash
   # Enter password
   ```

3. **Run a command and check logs**
   ```bash
   ls
   # Should see: extend command called
   # Should see: activity timestamp updated
   ```

4. **Check activity file updated**
   ```bash
   cat ~/.terminal_lock/activity.*.txt
   # Should show time of last command
   ```

5. **Expected Result:**
   - ✅ `terminal-lock extend` is called after each command
   - ✅ Activity timestamp is updated
   - ✅ Session is extended if still valid
   - ✅ No errors in logs

---

## Test 6: Edge Cases

### Edge Case 1: Session Expires During Command Execution

```bash
# At 8:59:50 (10 seconds before expiry)
# Type a long-running command
sleep 30

# Command starts at 8:59:50
# Session expires at 9:00:00 (during execution)
# Expected: Command completes normally
# Next prompt: Prompted for password
```

### Edge Case 2: Multiple Commands in Rapid Succession

```bash
# Type multiple commands quickly
ls && pwd && date && echo "test"

# Expected: All commands execute
# Activity timestamp: Updated once (after all commands)
# Session: Extended by 15 minutes
```

### Edge Case 3: Background Processes

```bash
# Start a background process
sleep 100 &

# Type another command
ls

# Expected: 
# - Background process continues
# - Foreground command executes
# - Activity timestamp updated
# - Session extended
```

---

## Debugging

### Check Session Status
```bash
terminal-lock status
```

### View Session File
```bash
cat ~/.terminal_lock/session.*.json
# Shows: expiry time and terminal ID
```

### View Activity File
```bash
cat ~/.terminal_lock/activity.*.txt
# Shows: last activity timestamp
```

### View All Lock Directory Contents
```bash
ls -la ~/.terminal_lock/
# Shows all session, activity, and lockout files
```

### Enable Debug Logging
```bash
RUST_LOG=debug terminal-lock auth
RUST_LOG=debug terminal-lock extend
```

### Reset Everything (for testing)
```bash
terminal-lock reset
# Clears all sessions, lockouts, and attempts
```

---

## Expected Behavior Summary

| Scenario | Before Fix | After Fix |
|----------|-----------|-----------|
| Open terminal | Prompt for password | Prompt for password |
| Session valid | Show greeting, freeze | Show greeting, shell continues |
| 15 min passes | Never re-checks | Prompts for password on next command |
| 5 min idle | Never detected | Prompts for password on next command |
| Run command | N/A | Activity updated, session extended |
| Multiple terminals | N/A | Each has independent session |

---

## Rollback (if needed)

If you need to revert the changes:

```bash
# Restore from backup
git checkout crates/cli/src/main.rs
git checkout crates/core/src/session.rs
git checkout crates/cli/src/commands/mod.rs
git checkout install.sh

# Rebuild
cargo build --release
```

---

## Success Criteria

All tests pass when:

1. ✅ Session expires exactly 15 minutes after creation
2. ✅ Idle session expires after 5 minutes of inactivity
3. ✅ Active session extends indefinitely (while in use)
4. ✅ Each terminal has independent session
5. ✅ Shell integration calls extend after each command
6. ✅ No infinite loops or shell freezing
7. ✅ All edge cases handled correctly
