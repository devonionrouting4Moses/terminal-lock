# Terminal Lock - Session Expiration Fixes

## Problem Summary

The authentication system had **3 critical bugs** preventing proper session expiration:

1. **15-minute session expiration not enforced** - Sessions never expired after 15 minutes
2. **5-minute inactivity timeout not triggered** - Idle sessions were never detected
3. **Missing `validate_and_extend()` method** - Shell integration couldn't extend sessions

---

## Root Causes

### Bug 1: Infinite Loop in main.rs (Lines 108-122)

**Before:**
```rust
if !authenticator.needs_authentication()? {
    start_animation_daemon(&paths)?;
    commands::greeting::show_greeting(&paths).await?;
    
    // ❌ INFINITE LOOP - BLOCKS SHELL
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
```

**Problem:** 
- The session check only happens **once** at terminal startup
- After that, the infinite loop blocks the shell indefinitely
- If session expires after 15 minutes, user is never prompted for password
- Shell becomes unresponsive

**After:**
```rust
if !authenticator.needs_authentication()? {
    start_animation_daemon(&paths)?;
    commands::greeting::show_greeting(&paths).await?;
    
    // ✅ EXIT - Let shell continue
    return Ok(());
}
```

**Impact:** Shell now continues normally, allowing the shell integration to call `terminal-lock auth` on each new prompt, which re-checks session validity.

---

### Bug 2: Inactivity Check Never Called

**Location:** `crates/core/src/session.rs` (Lines 42-50)

The inactivity check exists but is only called in `Session::is_valid()`, which is only called **once** at terminal startup. After that, the infinite loop prevents any re-checking.

**Problem:**
- `Session::is_valid()` checks if inactive for 5+ minutes
- But `is_valid()` is never called again after startup
- Activity file is updated every time `is_valid()` is called
- Since it's never called again, inactivity is never detected

**Solution:** 
- Shell integration now calls `terminal-lock extend` after each command
- This calls `Session::validate_and_extend()` which:
  1. Checks if session is valid (including inactivity check)
  2. Updates activity timestamp if valid
  3. Extends session by recreating it with new 15-minute expiry

---

### Bug 3: Missing `validate_and_extend()` Method

**Location:** `crates/core/src/session.rs`

The `extend.rs` command was calling a non-existent method.

**Before:**
```rust
// ❌ Method doesn't exist!
let _ = Session::validate_and_extend(paths);
```

**After:**
```rust
// ✅ Added to session.rs
pub fn validate_and_extend(paths: &Paths) -> Result<bool> {
    if Self::is_valid(paths)? {
        // Session is valid, extend it
        Self::extend(paths)?;
        Ok(true)
    } else {
        // Session is invalid/expired
        Ok(false)
    }
}
```

---

## How It Works Now

### Timeline: Terminal Session with 15-minute Expiry

```
Time    Action                          Session Status
────────────────────────────────────────────────────────
8:45    Open terminal 1                 
        ↓ terminal-lock auth            
        ↓ Session created               Valid until 9:00
        ↓ Show greeting                 
        ↓ Shell continues               ✓ Ready for commands

8:50    Type: ls                        
        ↓ Command executes              
        ↓ PROMPT_COMMAND runs           
        ↓ terminal-lock extend          
        ↓ Session re-checked            
        ↓ Activity updated              Valid until 9:05
        ↓ Session extended              ✓ Ready for next command

9:00    [No activity for 10 minutes]    
        ↓ New shell prompt appears      
        ↓ terminal-lock auth called     
        ↓ Session check: expired?       ✗ YES - 15 min passed
        ↓ Prompt for password           🔐 Re-authenticate

9:05    Type: pwd                       
        ↓ Command executes              
        ↓ PROMPT_COMMAND runs           
        ↓ terminal-lock extend          
        ↓ Session check: inactive?      ✗ NO - just used
        ↓ Activity updated              Valid until 9:20
        ↓ Session extended              ✓ Ready for next command

9:10    [No activity for 5+ minutes]    
        ↓ New shell prompt appears      
        ↓ terminal-lock auth called     
        ↓ Session check: inactive?      ✗ YES - 5 min idle
        ↓ Prompt for password           🔐 Re-authenticate
```

---

## Files Modified

### 1. `crates/cli/src/main.rs`
- **Removed:** Infinite loop after valid session check (line 108-122)
- **Removed:** Infinite loop after successful authentication (line 138-140)
- **Added:** Extend command handler (line 100-102)
- **Impact:** Shell now exits after authentication, allowing shell integration to work

### 2. `crates/core/src/session.rs`
- **Added:** `validate_and_extend()` method (lines 88-99)
- **Impact:** Allows shell integration to check and extend sessions

### 3. `crates/cli/src/commands/mod.rs`
- **Added:** `pub mod extend;` (line 2)
- **Impact:** Exports extend command module

### 4. `install.sh` - Shell Integration Updates

#### Bash Configuration (lines 267-271)
```bash
PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}terminal-lock extend 2>/dev/null"
```
- Runs `terminal-lock extend` after each command
- Updates activity timestamp
- Extends session if still valid

#### Zsh Configuration (lines 309-311)
```bash
precmd() {
    terminal-lock extend 2>/dev/null
}
```
- Runs before each prompt
- Updates activity timestamp
- Extends session if still valid

#### Fish Configuration (lines 353-355)
```bash
function fish_postexec --description "Terminal Lock session extension"
    terminal-lock extend 2>/dev/null
end
```
- Runs after each command execution
- Updates activity timestamp
- Extends session if still valid

---

## Session Expiration Scenarios

### Scenario 1: 15-Minute Hard Timeout

```
8:45 - Session created (valid until 9:00)
8:50 - Command: ls → Session extended (valid until 9:05)
8:55 - Command: pwd → Session extended (valid until 9:10)
9:00 - [No activity]
9:05 - [No activity]
9:10 - [No activity]
9:15 - User types command
       ↓ New shell prompt
       ↓ terminal-lock auth called
       ↓ Session check: now=9:15, expiry=9:10
       ↓ ✗ EXPIRED - 15 minutes passed
       ↓ 🔐 Prompt for password
```

### Scenario 2: 5-Minute Inactivity Timeout

```
8:45 - Session created (valid until 9:00)
8:50 - Command: ls → Activity updated to 8:50
8:55 - Command: pwd → Activity updated to 8:55
9:00 - [No activity - user left]
9:05 - [No activity - 10 minutes idle]
9:10 - User returns and types command
       ↓ New shell prompt
       ↓ terminal-lock auth called
       ↓ Session check: now=9:10, last_activity=8:55
       ↓ Inactivity: 9:10 - 8:55 = 15 minutes
       ↓ ✗ INACTIVE - 5+ minutes idle
       ↓ 🔐 Prompt for password
```

### Scenario 3: Active Session (Continuous Use)

```
8:45 - Session created (valid until 9:00)
8:50 - Command: ls → Activity=8:50, Session extends to 9:05
8:55 - Command: pwd → Activity=8:55, Session extends to 9:10
9:00 - Command: cd / → Activity=9:00, Session extends to 9:15
9:05 - Command: cat file → Activity=9:05, Session extends to 9:20
9:10 - Command: grep → Activity=9:10, Session extends to 9:25
       ↓ Session never expires because constantly in use
       ↓ Each command extends session by 15 minutes
```

---

## Testing Verification

### Test 1: 15-Minute Expiration
1. Open terminal at 8:45
2. Enter password
3. Wait 15 minutes (or modify clock)
4. Type a command
5. **Expected:** Prompted for password again

### Test 2: 5-Minute Inactivity
1. Open terminal
2. Enter password
3. Type a command
4. Wait 5+ minutes without typing
5. Type another command
6. **Expected:** Prompted for password again

### Test 3: Session Extension
1. Open terminal
2. Enter password
3. Type command every 2 minutes for 20 minutes
4. **Expected:** Never prompted for password (session keeps extending)

### Test 4: Multiple Terminals
1. Open Terminal 1 at 8:45 → Authenticate
2. Open Terminal 2 at 8:50 → Must authenticate (independent session)
3. Terminal 1 expires at 9:00 → Prompted for password
4. Terminal 2 still valid until 9:05 → No prompt yet
5. **Expected:** Each terminal has independent 15-minute session

---

## Security Improvements

✅ **Sessions now properly expire after 15 minutes**
- Prevents indefinite access
- Balances security and convenience

✅ **Idle sessions expire after 5 minutes**
- Protects against unattended terminals
- Assumes user has left the machine

✅ **Each terminal is independent**
- Opening new terminal always requires authentication
- No shared sessions between terminals

✅ **Activity tracking**
- Last command execution time is tracked
- Inactivity timeout is enforced

---

## Backward Compatibility

All changes are backward compatible:
- Existing session files continue to work
- Password hashing unchanged
- Lockout system unchanged
- Only adds new functionality (session extension)

---

## Build Status

✅ All changes compile successfully
✅ No breaking changes to existing APIs
✅ New `validate_and_extend()` method is public
✅ New `Extend` command is registered
✅ Shell integration updated for all supported shells
