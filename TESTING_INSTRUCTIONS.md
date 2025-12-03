# Testing Instructions - Terminal Lock Fixes

## Quick Start

### Prerequisites
- Binary is compiled and installed at `~/.local/bin/terminal-lock`
- Shell configuration updated with new `check` command
- All session files cleaned up

### Clean Slate
```bash
rm -rf ~/.terminal_lock/session.*.json
```

---

## Test 1: Lock Invalidation

### Objective
Verify that sessions are invalidated when the laptop is locked, and the terminal becomes unusable.

### Steps

#### Step 1: Clean Up
```bash
rm -rf ~/.terminal_lock/session.*.json
ls -la ~/.terminal_lock/session.*.json  # Should show: no matches
```

#### Step 2: Open New Terminal & Authenticate
1. Open a **brand new terminal window**
2. You should be prompted: `🔐 Enter terminal password:`
3. Enter your password
4. You should see greeting and quote

#### Step 3: Verify Session Created
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show ONE file: session.XXXXX.json (where XXXXX is your terminal PID)
```

#### Step 4: Run Commands
Execute these commands to verify terminal works:
```bash
ls
pwd
whoami
uname -r
date
```

#### Step 5: Lock Your Laptop
- Press **Super+L** (or your system's lock shortcut)
- Wait for screen to fully lock
- You should see in terminal: `🔒 System lock detected via D-Bus`

#### Step 6: Verify Session Deleted
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show: no matches (session file was deleted)
```

#### Step 7: Unlock Laptop
- Unlock your laptop
- Return to the same terminal window

#### Step 8: Try to Run Command
```bash
ls
```

#### Expected Result
❌ **Terminal should be BLOCKED**
- You should see: `❌ Session expired or invalid. Please authenticate again.`
- Command should NOT execute
- Terminal should be read-only

#### If This Works ✅
Lock invalidation is working correctly!

---

## Test 2: Session Cleanup on Re-authentication

### Objective
Verify that old session files are deleted when creating a new session (e.g., after `source ~/.zshrc`).

### Steps

#### Step 1: Clean Up
```bash
rm -rf ~/.terminal_lock/session.*.json
```

#### Step 2: Open New Terminal & Authenticate
1. Open a **brand new terminal window**
2. Authenticate with your password
3. Note the PID shown in debug output (e.g., 644939)

#### Step 3: Verify One Session Exists
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show ONE file: session.644939.json
```

#### Step 4: Run Some Commands
```bash
ls
pwd
whoami
```

#### Step 5: Refresh Shell Configuration
```bash
source ~/.zshrc
```

You should see:
- `🔐 Enter terminal password:` prompt again
- Authentication successful message
- New greeting and quote
- New PID in debug output (e.g., 646091)

#### Step 6: Verify Still Only One Session
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show ONE file: session.646091.json (NEW PID, old one deleted)
# NOT two files!
```

#### Step 7: Run Commands in Refreshed Shell
```bash
ls
pwd
whoami
```

#### Expected Result
✅ **Only ONE session file should exist**
- Old session (644939) should be deleted
- New session (646091) should be created
- No duplicate session files

#### If This Works ✅
Session cleanup is working correctly!

---

## Test 3: Automated Tests

### Run Lock Invalidation Test
```bash
./test_lock_invalidation.sh
```

**Expected Output**:
```
═══════════════════════════════════════════════════════════
Lock Invalidation Test
═══════════════════════════════════════════════════════════

✓ No existing sessions found
✓ Check correctly failed (exit code 1) - terminal would be blocked
✓ Session created: session.XXXXX.json
✓ Check succeeded (exit code 0) - terminal is usable
✓ Session deleted (simulating lock detection)
✓ Check correctly failed (exit code 1) - terminal is now blocked

═══════════════════════════════════════════════════════════
Test Results
═══════════════════════════════════════════════════════════

✓ All tests passed!
```

### Run Session Cleanup Test
```bash
./test_session_cleanup.sh
```

**Expected Output**:
```
═══════════════════════════════════════════════════════════
Session Cleanup Test
═══════════════════════════════════════════════════════════

ℹ Step 1: Cleaning up existing sessions...
✓ Cleaned up old sessions
ℹ Step 2: Creating first session (simulating initial authentication)...
✓ First session created: session.XXXXX.json
ℹ Step 3: Simulating 'source ~/.zshrc' (re-authentication in subshell)...
ℹ Step 4: Checking that only ONE session file exists
✓ Only 1 session file exists after re-authentication
ℹ Step 5: Verifying the old session was deleted
✓ Old session was replaced with new session

═══════════════════════════════════════════════════════════
Test Results
═══════════════════════════════════════════════════════════

✓ All tests passed!
```

---

## Test 4: Multiple Terminals

### Objective
Verify that each terminal has its own independent session.

### Steps

#### Step 1: Clean Up
```bash
rm -rf ~/.terminal_lock/session.*.json
```

#### Step 2: Open Terminal 1
1. Open first terminal window
2. Authenticate with password
3. Note the PID (e.g., 644939)
4. Run a command: `ls`

#### Step 3: Verify Terminal 1 Session
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show: session.644939.json
```

#### Step 4: Open Terminal 2
1. Open second terminal window (different from Terminal 1)
2. You should be prompted to authenticate (separate session)
3. Authenticate with password
4. Note the PID (e.g., 645000 - different from Terminal 1)
5. Run a command: `pwd`

#### Step 5: Verify Both Sessions Exist
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show TWO files:
# session.644939.json (Terminal 1)
# session.645000.json (Terminal 2)
```

#### Step 6: Lock Laptop
- Press Super+L to lock

#### Step 7: Verify Both Sessions Deleted
```bash
ls -la ~/.terminal_lock/session.*.json
# Should show: no matches (both deleted)
```

#### Step 8: Unlock Laptop

#### Step 9: Try Commands in Both Terminals
- In Terminal 1: `ls` → should be blocked
- In Terminal 2: `pwd` → should be blocked

#### Expected Result
✅ **Both terminals blocked after lock**
- Each terminal had independent session
- Lock invalidated all sessions
- Both terminals require re-authentication

#### If This Works ✅
Multi-terminal isolation is working correctly!

---

## Troubleshooting

### Issue: Terminal not blocked after lock
**Possible Causes**:
1. D-Bus not detecting lock event
2. Session file not being deleted
3. Shell integration not calling `check` command

**Debug**:
```bash
# Check if D-Bus is working
dbus-send --session --print-reply --dest=org.gnome.SessionManager /org/gnome/SessionManager/Presence org.freedesktop.DBus.Properties.Get string:"org.gnome.SessionManager.Presence" string:"status"

# Check if session file exists
ls -la ~/.terminal_lock/session.*.json

# Check shell integration
grep "terminal-lock check" ~/.zshrc
```

### Issue: Multiple session files after `source ~/.zshrc`
**Possible Causes**:
1. Binary not updated with cleanup code
2. Shell not sourcing updated config

**Debug**:
```bash
# Verify binary is updated
terminal-lock --version

# Check if cleanup is happening
source ~/.zshrc 2>&1 | grep -i "session\|debug"

# Count session files
ls ~/.terminal_lock/session.*.json | wc -l
```

### Issue: Authentication prompt appears twice
**Possible Causes**:
1. Shell integration running twice
2. Multiple precmd functions

**Debug**:
```bash
# Check for duplicate entries
grep -c "terminal-lock auth" ~/.zshrc

# Remove duplicates if found
# Edit ~/.zshrc and remove duplicate blocks
```

---

## Success Criteria

### ✅ Lock Invalidation Working
- [ ] Session created on authentication
- [ ] Lock detected when laptop locked
- [ ] Session file deleted on lock
- [ ] Terminal blocked after lock
- [ ] Re-authentication required

### ✅ Session Cleanup Working
- [ ] One session file after initial auth
- [ ] One session file after `source ~/.zshrc`
- [ ] Old session file deleted
- [ ] New session file created
- [ ] No duplicate sessions

### ✅ Multi-Terminal Working
- [ ] Each terminal has independent session
- [ ] Lock invalidates all sessions
- [ ] Each terminal requires re-auth after lock

### ✅ Overall Security
- [ ] 15-minute hard timeout enforced
- [ ] 5-minute inactivity timeout enforced
- [ ] Per-terminal session isolation maintained
- [ ] No session extension on commands

---

## Next Steps

Once all tests pass:
1. Document any issues found
2. Verify behavior matches expectations
3. Test in real-world usage scenarios
4. Consider additional edge cases
5. Update documentation if needed

---

## Support

If you encounter any issues:
1. Check the troubleshooting section above
2. Review the debug output
3. Check the documentation files:
   - `LOCK_INVALIDATION_FIX.md`
   - `SESSION_CLEANUP_IMPLEMENTATION.md`
   - `COMPLETE_FIX_SUMMARY.md`
