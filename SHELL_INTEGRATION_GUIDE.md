# Shell Integration Guide - Cross-Platform Terminal Lock

## Overview
Terminal Lock integrates with all major shells across Linux, macOS, and Windows to enforce session validation and block terminal access when sessions are invalid.

---

## Supported Shells

### Linux & macOS
- ✅ **Bash** - POSIX shell (default on many systems)
- ✅ **Zsh** - Modern shell (macOS default since Catalina)
- ✅ **Fish** - User-friendly shell

### Windows
- ✅ **PowerShell** - Windows native shell
- ✅ **Git Bash** - Bash on Windows (via Git for Windows)
- ✅ **WSL (Windows Subsystem for Linux)** - Full Linux environment

---

## Integration Strategy

### How It Works

Each shell is configured to:

1. **On Shell Startup** (Interactive Mode Only)
   - Check if password is configured
   - If not configured: Run setup wizard
   - If configured: Check for valid session
   - If no valid session: Prompt for authentication
   - If authentication fails: Exit shell

2. **Before Each Command** (Session Validation)
   - Check if current session is still valid
   - If invalid: Exit shell with message
   - If valid: Allow command execution
   - After command: Update activity timestamp

3. **Session Timeout Enforcement**
   - 15-minute hard expiry (no extension)
   - 5-minute inactivity timeout
   - Lock detection invalidates all sessions

---

## Bash Integration

### Location
`~/.bashrc` (Linux/macOS) or `~/.bash_profile` (macOS)

### Configuration
```bash
# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if [[ $- == *i* ]]; then
    if command -v terminal-lock &> /dev/null; then
        terminal-lock auth 2>&1
        if [ $? -ne 0 ]; then
            echo "❌ Authentication failed"
            exit 1
        fi
    fi
fi

# ============================================
# Terminal Lock - Session Validation & Extension
# Checks session validity BEFORE showing prompt
# Blocks terminal if session is invalid
# ============================================
PROMPT_COMMAND="terminal-lock check 2>/dev/null || exit 1; terminal-lock extend 2>/dev/null; ${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}"
```

### How It Works
- `PROMPT_COMMAND` runs before each prompt is displayed
- `terminal-lock check` validates session (exit code 0 = valid, 1 = invalid)
- `|| exit 1` exits shell if check fails
- `terminal-lock extend` updates activity timestamp
- Semicolon separates commands

### Behavior
```
User opens terminal
  ↓
Shell runs auth check
  ↓
If no session: Prompt for password
  ↓
If password correct: Create session
  ↓
Show prompt
  ↓
User types command
  ↓
PROMPT_COMMAND runs (before showing next prompt)
  ↓
Check session validity
  ↓
If invalid: Exit shell
If valid: Update activity, show prompt
```

---

## Zsh Integration

### Location
`~/.zshrc`

### Configuration
```zsh
# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if [[ -o interactive && -t 0 ]]; then
    if command -v terminal-lock &> /dev/null; then
        terminal-lock auth 2>&1
        if [ $? -ne 0 ]; then
            echo "❌ Authentication failed"
            exit 1
        fi
    fi
fi

# ============================================
# Terminal Lock - Session Validation & Extension
# Checks session validity BEFORE showing prompt
# Blocks terminal if session is invalid
# ============================================
zle-line-init() {
    # Check session validity BEFORE showing prompt
    # This prevents user from typing commands if session is invalid
    terminal-lock check 2>/dev/null
    if [ $? -ne 0 ]; then
        # Session invalid - exit shell to force re-authentication
        echo ""
        echo "🔐 Session expired. Please authenticate again."
        exit 1
    fi
    
    # Session is valid - show prompt normally
    zle reset-prompt
}

zle -N zle-line-init

# Update activity timestamp after each command
precmd() {
    terminal-lock extend 2>/dev/null
}
```

### How It Works
- `zle-line-init` hook runs when prompt is about to be shown
- Checks session validity BEFORE user can type
- If invalid: Exits shell immediately
- If valid: Shows prompt normally
- `precmd` hook runs after command execution
- Updates activity timestamp

### Behavior
```
User opens terminal
  ↓
Shell runs auth check
  ↓
If no session: Prompt for password
  ↓
If password correct: Create session
  ↓
zle-line-init runs
  ↓
Check session validity
  ↓
If invalid: Exit shell
If valid: Show prompt
  ↓
User types command
  ↓
Command executes
  ↓
precmd runs
  ↓
Update activity timestamp
  ↓
zle-line-init runs again
  ↓
Check session validity again
```

---

## Fish Integration

### Location
`~/.config/fish/config.fish`

### Configuration
```fish
# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if status is-interactive
    if command -v terminal-lock > /dev/null
        terminal-lock auth
        if test $status -ne 0
            echo "❌ Authentication failed"
            exit 1
        end
    end
end

# ============================================
# Terminal Lock - Session Validation & Extension
# Checks session validity BEFORE showing prompt
# Blocks terminal if session is invalid
# ============================================
function fish_postexec --description "Terminal Lock session validation and extension"
    # Check session validity after each command
    terminal-lock check 2>/dev/null
    if test $status -ne 0
        echo ""
        echo "🔐 Session expired. Please authenticate again."
        exit 1
    end
    
    # Update activity timestamp
    terminal-lock extend 2>/dev/null
end

# Add to PATH
set -gx PATH $PATH $HOME/.local/bin
```

### How It Works
- `fish_postexec` hook runs after each command execution
- Checks session validity
- If invalid: Exits shell
- If valid: Updates activity timestamp
- Fish automatically shows prompt after hook completes

### Behavior
```
User opens terminal
  ↓
Shell runs auth check
  ↓
If no session: Prompt for password
  ↓
If password correct: Create session
  ↓
Show prompt
  ↓
User types command
  ↓
Command executes
  ↓
fish_postexec runs
  ↓
Check session validity
  ↓
If invalid: Exit shell
If valid: Update activity, show prompt
```

---

## PowerShell Integration (Windows)

### Location
`$PROFILE` (typically `%USERPROFILE%\Documents\PowerShell\profile.ps1`)

### Configuration
```powershell
# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if ($null -ne (Get-Command terminal-lock -ErrorAction SilentlyContinue)) {
    & terminal-lock auth
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Authentication failed" -ForegroundColor Red
        exit 1
    }
}

# ============================================
# Terminal Lock - Session Validation & Extension
# Checks session validity BEFORE showing prompt
# Blocks terminal if session is invalid
# ============================================
function prompt {
    # Check session validity before showing prompt
    & terminal-lock check 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "🔐 Session expired. Please authenticate again." -ForegroundColor Yellow
        exit 1
    }
    
    # Update activity timestamp
    & terminal-lock extend 2>$null
    
    # Show prompt
    "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
}
```

### How It Works
- `prompt` function is called before each prompt display
- Checks session validity
- If invalid: Exits shell
- If valid: Updates activity and shows prompt

---

## Git Bash Integration (Windows)

### Location
`~/.bashrc` (in Git Bash home directory)

### Configuration
Same as Bash integration above (uses POSIX bash syntax)

---

## WSL Integration (Windows Subsystem for Linux)

### Location
`~/.bashrc` or `~/.zshrc` (depending on shell)

### Configuration
Same as Linux integration above

---

## Installation

### Automatic Installation
Run the installer script:
```bash
./install.sh
```

This will:
1. Detect your shell(s)
2. Backup existing configuration
3. Add Terminal Lock integration
4. Configure PATH

### Manual Installation

#### Bash
Add to `~/.bashrc`:
```bash
# Copy the Bash configuration from above
```

#### Zsh
Add to `~/.zshrc`:
```zsh
# Copy the Zsh configuration from above
```

#### Fish
Add to `~/.config/fish/config.fish`:
```fish
# Copy the Fish configuration from above
```

#### PowerShell
Add to `$PROFILE`:
```powershell
# Copy the PowerShell configuration from above
```

---

## Testing Shell Integration

### Test Bash
```bash
bash
# Should prompt for password if no session
# Should show greeting after authentication
```

### Test Zsh
```bash
zsh
# Should prompt for password if no session
# Should show greeting after authentication
```

### Test Fish
```bash
fish
# Should prompt for password if no session
# Should show greeting after authentication
```

### Test PowerShell (Windows)
```powershell
pwsh
# Should prompt for password if no session
# Should show greeting after authentication
```

---

## Session Validation Behavior

### Valid Session
```
User opens terminal
  ↓
Session exists and is valid (< 15 min, < 5 min inactive)
  ↓
Shell shows prompt immediately
  ↓
User can type commands
```

### Expired Session
```
User opens terminal
  ↓
Session expired (> 15 min)
  ↓
Shell prompts for password
  ↓
User authenticates
  ↓
New session created
  ↓
Shell shows prompt
```

### Inactive Session
```
User opens terminal
  ↓
Session inactive (> 5 min)
  ↓
Shell prompts for password
  ↓
User authenticates
  ↓
New session created
  ↓
Shell shows prompt
```

### Locked System
```
User opens terminal and authenticates
  ↓
Session created
  ↓
User locks laptop (Super+L)
  ↓
D-Bus detects lock
  ↓
All sessions deleted
  ↓
User unlocks laptop
  ↓
User tries to use terminal
  ↓
Shell checks session (invalid)
  ↓
Shell exits with message
  ↓
User must re-authenticate
```

---

## Cross-Platform Compatibility

### Linux
- ✅ Bash
- ✅ Zsh
- ✅ Fish
- ✅ D-Bus lock detection

### macOS
- ✅ Bash (legacy)
- ✅ Zsh (default)
- ✅ Fish
- ✅ macOS lock detection

### Windows
- ✅ PowerShell
- ✅ Git Bash
- ✅ WSL (full Linux support)
- ✅ Windows lock detection

---

## Troubleshooting

### Shell not prompting for password
**Solution**: Check if `terminal-lock` is in PATH
```bash
which terminal-lock
```

### Multiple authentication prompts
**Solution**: Check for duplicate configurations
```bash
grep -c "terminal-lock auth" ~/.zshrc
```

### Terminal not blocking after lock
**Solution**: Verify session validation is working
```bash
terminal-lock check
echo $?  # Should be 1 if no session
```

### Shell integration not loading
**Solution**: Source the configuration file
```bash
source ~/.zshrc  # For Zsh
source ~/.bashrc  # For Bash
```

---

## Summary

Terminal Lock provides consistent session management across all major shells and platforms:

| Shell | Linux | macOS | Windows |
|-------|-------|-------|---------|
| Bash | ✅ | ✅ | ✅ (Git Bash) |
| Zsh | ✅ | ✅ | ✅ (WSL) |
| Fish | ✅ | ✅ | ✅ (WSL) |
| PowerShell | ❌ | ❌ | ✅ |

All shells enforce:
- ✅ Per-terminal authentication
- ✅ 15-minute hard timeout
- ✅ 5-minute inactivity timeout
- ✅ Lock detection and invalidation
- ✅ Terminal blocking on invalid session
