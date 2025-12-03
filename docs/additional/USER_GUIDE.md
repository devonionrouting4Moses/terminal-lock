# Terminal Lock - Complete User Guide

**Version 1.0** | Enterprise-Grade Terminal Authentication System

---

## 🎯 Executive Summary

Terminal Lock is a daemon-based terminal authentication system that provides **military-grade security** for your command-line environment across **Linux, macOS, and Windows**. Each terminal session is independently authenticated, tracked, and secured with **automatic lock detection**, **session expiration**, and **intelligent threat mitigation**.

### Key Features at a Glance

| Feature | Description | Benefit |
|---------|-------------|---------|
| 🔐 **Per-Terminal Authentication** | Each terminal window requires independent authentication | Prevents unauthorized terminal access even on unlocked machines |
| ⏱️ **15-Minute Hard Timeout** | Sessions expire after 15 minutes regardless of activity | Ensures credentials aren't left exposed |
| 💤 **5-Minute Idle Detection** | Auto-locks terminals after 5 minutes of inactivity | Protects against walk-away attacks |
| 🔒 **System Lock Integration** | Detects OS-level screen locks and invalidates all sessions | Synchronizes with system security |
| 🗃️ **SQLite Session Storage** | Persistent, ACID-compliant session database | Reliable session tracking across crashes |
| 🚀 **Daemon Architecture** | Background service manages all sessions | Fast, efficient, crash-resistant |
| 🌐 **Cross-Platform Support** | Linux (all distros), macOS, Windows | Unified security across your entire infrastructure |
| 🔄 **Multi-Terminal Isolation** | Terminal 1's session never affects Terminal 2 | True session independence |
| 🎨 **Beautiful UI** | Matrix animations, custom themes, inspirational quotes | Security doesn't have to be boring |
| 📊 **Comprehensive Logging** | Detailed audit logs of all authentication events | Forensics and compliance |

---

## 📐 Architecture Overview

### System Design

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER LAYER                              │
├─────────────────────────────────────────────────────────────────┤
│  Terminal 1     Terminal 2     Terminal 3     Terminal N...     │
│  (zsh/bash)     (fish)         (PowerShell)   (any shell)       │
│     │               │               │               │            │
│     └───────────────┴───────────────┴───────────────┘            │
│                         │                                        │
│                         ▼                                        │
│              ┌──────────────────────┐                           │
│              │   Shell Hook (CLI)   │                           │
│              │  terminal-lock       │                           │
│              └──────────────────────┘                           │
│                         │                                        │
│                         ▼                                        │
├─────────────────────────────────────────────────────────────────┤
│                    IPC LAYER (Unix Socket / Named Pipe)         │
├─────────────────────────────────────────────────────────────────┤
│                         │                                        │
│                         ▼                                        │
│              ┌──────────────────────┐                           │
│              │  Session Daemon      │                           │
│              │  (Background Service)│                           │
│              │                      │                           │
│              │  - Session Manager   │                           │
│              │  - Auth Handler      │                           │
│              │  - Lock Monitor      │                           │
│              │  - Timeout Enforcer  │                           │
│              └──────────────────────┘                           │
│                         │                                        │
│                         ▼                                        │
├─────────────────────────────────────────────────────────────────┤
│                    STORAGE LAYER                                │
├─────────────────────────────────────────────────────────────────┤
│                         │                                        │
│         ┌───────────────┴───────────────┐                       │
│         ▼                               ▼                       │
│  ┌─────────────┐              ┌──────────────┐                 │
│  │   SQLite    │              │   System     │                 │
│  │  Database   │              │   Monitor    │                 │
│  │             │              │              │                 │
│  │ sessions    │              │ D-Bus/WMI    │                 │
│  │ users       │              │ Lock Events  │                 │
│  │ audit_log   │              │              │                 │
│  └─────────────┘              └──────────────┘                 │
└─────────────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. CLI Tool (`terminal-lock`)
**Location**: `/usr/local/bin/terminal-lock` (Unix) or `C:\Program Files\terminal-lock\terminal-lock.exe` (Windows)

**Purpose**: User-facing interface for all authentication operations

**Commands**:
```bash
terminal-lock auth          # Authenticate current terminal
terminal-lock check         # Validate session (called by hooks)
terminal-lock extend        # Update activity timestamp
terminal-lock status        # Show session information
terminal-lock lock          # Manually lock current terminal
terminal-lock daemon start  # Start background daemon
terminal-lock init          # Generate shell hook
terminal-lock reset         # Clear all sessions (admin)
```

#### 2. Session Daemon (`terminal-lockd`)
**Location**: 
- Linux: `/usr/lib/systemd/user/terminal-lock.service`
- macOS: `~/Library/LaunchAgents/com.terminal-lock.daemon.plist`
- Windows: Windows Service Registry

**Purpose**: Central session management service that runs in the background

**Responsibilities**:
- ✅ Manage SQLite database of active sessions
- ✅ Handle authentication requests from CLI
- ✅ Validate session tokens
- ✅ Enforce timeout policies (15min hard, 5min idle)
- ✅ Monitor system lock events (D-Bus/WMI)
- ✅ Generate lock generation counters
- ✅ Maintain audit logs
- ✅ Handle crash recovery

**Communication**: IPC via Unix domain sockets (Linux/macOS) or Named Pipes (Windows)

#### 3. SQLite Database
**Location**: 
- Linux: `~/.local/share/terminal-lock/sessions.db`
- macOS: `~/Library/Application Support/terminal-lock/sessions.db`
- Windows: `%APPDATA%\terminal-lock\sessions.db`

**Schema**:
```sql
-- Active sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,              -- UUID v4
    user_id INTEGER NOT NULL,         -- System UID
    shell_pid INTEGER NOT NULL,       -- Shell process ID
    created_at INTEGER NOT NULL,      -- Unix timestamp
    expires_at INTEGER NOT NULL,      -- Unix timestamp (created + 15min)
    last_activity INTEGER NOT NULL,   -- Unix timestamp
    status TEXT NOT NULL,             -- Active/Paused/Expired/Locked
    lock_generation INTEGER NOT NULL, -- Increments on each system lock
    terminal_type TEXT,               -- zsh/bash/fish/powershell
    display TEXT                      -- X11 DISPLAY or TTY
);

-- User configuration
CREATE TABLE users (
    uid INTEGER PRIMARY KEY,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    failed_attempts INTEGER DEFAULT 0,
    locked_until INTEGER,
    preferences TEXT                  -- JSON blob
);

-- Audit log
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,         -- AUTH_SUCCESS/AUTH_FAIL/LOCK/UNLOCK
    user_id INTEGER NOT NULL,
    shell_pid INTEGER,
    session_id TEXT,
    details TEXT                      -- JSON blob
);

-- Lock generation tracking
CREATE TABLE lock_generation (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    generation INTEGER NOT NULL DEFAULT 0,
    last_lock_time INTEGER
);
```

#### 4. Lock Monitor
**Purpose**: Real-time detection of system lock events

**Platform Implementations**:

**Linux (D-Bus)**:
```rust
// Listens to org.gnome.SessionManager.Presence
// Status 0 = Available (unlocked)
// Status 1 = Invisible (LOCKED)
// Status 2 = Busy
// Status 3 = Idle

// Also monitors:
// - org.freedesktop.ScreenSaver.ActiveChanged
// - loginctl LockedHint
// - systemd lock signals
```

**macOS (Distributed Notifications)**:
```rust
// Monitors:
// - com.apple.screenIsLocked
// - com.apple.screenIsUnlocked
// - NSWorkspaceScreensDidSleepNotification
// - IORegisterForSystemPower callbacks
```

**Windows (WTS API)**:
```rust
// Monitors:
// - WTS_SESSION_LOCK
// - WTS_SESSION_UNLOCK
// - WM_WTSSESSION_CHANGE messages
```

---

## 🚀 Installation

### Linux (Ubuntu/Debian/Pop!_OS/Mint)

```bash
# Download latest release
curl -sSL https://github.com/yourusername/terminal-lock/releases/latest/download/terminal-lock-linux-amd64.tar.gz -o terminal-lock.tar.gz

# Extract
tar xzf terminal-lock.tar.gz
cd terminal-lock

# Install binaries
sudo ./install.sh

# Or manual installation:
sudo cp target/release/terminal-lock /usr/local/bin/
sudo cp target/release/terminal-lockd /usr/local/bin/
sudo chmod +x /usr/local/bin/terminal-lock*

# Install systemd service
mkdir -p ~/.config/systemd/user
cp systemd/terminal-lock.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable terminal-lock
systemctl --user start terminal-lock

# Verify daemon is running
systemctl --user status terminal-lock

# Add shell hook (choose your shell)
# For zsh:
terminal-lock init >> ~/.zshrc
source ~/.zshrc

# For bash:
terminal-lock init >> ~/.bashrc
source ~/.bashrc

# For fish:
terminal-lock init fish >> ~/.config/fish/config.fish
```

### Linux (Fedora/RHEL/CentOS)

```bash
# Download RPM package
curl -sSL https://github.com/yourusername/terminal-lock/releases/latest/download/terminal-lock-1.0.x86_64.rpm -o terminal-lock.rpm

# Install
sudo dnf install ./terminal-lock.rpm

# Or using yum:
sudo yum localinstall ./terminal-lock.rpm

# Enable and start service
systemctl --user enable --now terminal-lock

# Add shell hook
terminal-lock init >> ~/.bashrc
source ~/.bashrc
```

### Linux (Arch/Manjaro)

```bash
# Install from AUR
yay -S terminal-lock-git

# Or manual:
git clone https://aur.archlinux.org/terminal-lock-git.git
cd terminal-lock-git
makepkg -si

# Enable service
systemctl --user enable --now terminal-lock

# Add hook
terminal-lock init >> ~/.zshrc
source ~/.zshrc
```

### macOS

```bash
# Using Homebrew
brew tap yourusername/terminal-lock
brew install terminal-lock

# Or download DMG
curl -sSL https://github.com/yourusername/terminal-lock/releases/latest/download/terminal-lock-macos.dmg -o terminal-lock.dmg
open terminal-lock.dmg
# Drag to Applications

# Install LaunchAgent
terminal-lock daemon install

# Add shell hook
terminal-lock init >> ~/.zshrc
source ~/.zshrc
```

### Windows

```powershell
# Download installer
Invoke-WebRequest -Uri "https://github.com/yourusername/terminal-lock/releases/latest/download/terminal-lock-windows-setup.exe" -OutFile "terminal-lock-setup.exe"

# Run installer (GUI)
.\terminal-lock-setup.exe

# Or using Chocolatey
choco install terminal-lock

# Or using Scoop
scoop bucket add terminal-lock https://github.com/yourusername/scoop-terminal-lock
scoop install terminal-lock

# Service is automatically installed and started

# Add to PowerShell profile
terminal-lock init powershell >> $PROFILE
. $PROFILE

# Add to Git Bash (if using)
terminal-lock init bash >> ~/.bashrc
```

---

## 💻 Multi-User Scenarios

### Scenario 1: Single User, Multiple Terminals

**User**: Alice (UID 1000)
**Machine**: Personal laptop running Ubuntu 22.04
**Terminals**: 3 terminal windows open

```
┌─────────────────────────────────────────────────────────┐
│ Alice's Desktop (Unlocked)                              │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Terminal 1 (PID 12345)          Terminal 2 (PID 23456) │
│  ┌────────────────────┐          ┌────────────────────┐ │
│  │ Session: a1b2c3d4  │          │ Session: e5f6g7h8  │ │
│  │ Status: Active     │          │ Status: Active     │ │
│  │ Expires: 14:45     │          │ Expires: 14:50     │ │
│  │ $ vim main.rs      │          │ $ cargo build      │ │
│  └────────────────────┘          └────────────────────┘ │
│                                                          │
│  Terminal 3 (PID 34567)                                 │
│  ┌────────────────────┐                                 │
│  │ Session: i9j0k1l2  │                                 │
│  │ Status: Active     │                                 │
│  │ Expires: 14:55     │                                 │
│  │ $ git commit       │                                 │
│  └────────────────────┘                                 │
└─────────────────────────────────────────────────────────┘

SQLite Database State:
╔═══════════╦════════════╦═════════╦═══════════╦══════════╗
║ SessionID ║ Shell PID  ║ Status  ║ Created   ║ Expires  ║
╠═══════════╬════════════╬═════════╬═══════════╬══════════╣
║ a1b2c3d4  ║ 12345      ║ Active  ║ 14:30:00  ║ 14:45:00 ║
║ e5f6g7h8  ║ 23456      ║ Active  ║ 14:35:00  ║ 14:50:00 ║
║ i9j0k1l2  ║ 34567      ║ Active  ║ 14:40:00  ║ 14:55:00 ║
╚═══════════╩════════════╩═════════╩═══════════╩══════════╝
```

**Timeline**:

```
14:30:00 - Terminal 1 opens
         → User authenticates (password: ******)
         → Session a1b2c3d4 created (expires 14:45:00)
         → Database: INSERT INTO sessions...

14:35:00 - Terminal 2 opens (within same session window)
         → User authenticates (password: ******)
         → Session e5f6g7h8 created (expires 14:50:00)
         → Database: INSERT INTO sessions...
         → Terminal 1 still valid (10 min remaining)

14:40:00 - Terminal 3 opens
         → User authenticates
         → Session i9j0k1l2 created (expires 14:55:00)
         → Terminals 1 & 2 still valid

14:43:00 - User types in Terminal 1
         → Hook calls: terminal-lock extend
         → CLI → IPC → Daemon
         → Daemon updates last_activity timestamp
         → Session still expires at 14:45 (hard limit)

14:45:00 - Terminal 1 session expires
         → User tries to run command
         → Hook calls: terminal-lock check
         → Daemon: Session expired (15 min hard limit)
         → Terminal 1 blocked
         → User sees: "🔐 Session expired. Re-authenticate."
         → User enters password again
         → New session created

14:50:00 - Terminal 2 expires
14:55:00 - Terminal 3 expires
```

### Scenario 2: System Lock Event

**Timeline**:
```
15:00:00 - All 3 terminals authenticated and active
         → Lock generation = 5

15:05:00 - User presses Super+L (locks screen)
         → Lock Monitor detects: D-Bus signal received
         → Daemon: INCREMENT lock_generation (5 → 6)
         → Daemon: UPDATE sessions SET status='Paused'
         → Database state:
           ╔═══════════╦═════════╦══════════╦════════════╗
           ║ SessionID ║ Status  ║ LockGen  ║ Action     ║
           ╠═══════════╬═════════╬══════════╬════════════╣
           ║ a1b2c3d4  ║ Paused  ║ 5        ║ Invalidate ║
           ║ e5f6g7h8  ║ Paused  ║ 5        ║ Invalidate ║
           ║ i9j0k1l2  ║ Paused  ║ 5        ║ Invalidate ║
           ╚═══════════╩═════════╩══════════╩════════════╝
         → All sessions now invalid

15:10:00 - User unlocks laptop (enters system password)
         → Lock Monitor detects: D-Bus unlock signal
         → Daemon: Log unlock event
         → Sessions remain Paused (require re-auth)

15:10:30 - User tries to use Terminal 1
         → Hook calls: terminal-lock check
         → Daemon: Session lock_generation (5) < current (6)
         → Response: SessionInvalid (LockGenerationMismatch)
         → Terminal 1 shows: "🔐 System was locked. Re-authenticate."
         → User enters password
         → New session created with lock_generation = 6

15:11:00 - User tries Terminal 2
         → Same flow - requires re-authentication
         → New session created

15:12:00 - User tries Terminal 3
         → Same flow - requires re-authentication
         → New session created
```

### Scenario 3: Multi-User Server Environment

**Machine**: Ubuntu Server 22.04
**Users**: Alice (UID 1000), Bob (UID 1001), Charlie (UID 1002)
**Scenario**: Multiple users SSH'd into same server

```
┌─────────────────────────────────────────────────────────┐
│ Server: production.company.com                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│ Alice (UID 1000)            Bob (UID 1001)              │
│ ┌─────────────────┐         ┌─────────────────┐        │
│ │ SSH Session 1   │         │ SSH Session 1   │        │
│ │ PID 10001       │         │ PID 20001       │        │
│ │ Session: aaa111 │         │ Session: bbb222 │        │
│ └─────────────────┘         └─────────────────┘        │
│ ┌─────────────────┐         ┌─────────────────┐        │
│ │ SSH Session 2   │         │ SSH Session 2   │        │
│ │ PID 10002       │         │ PID 20002       │        │
│ │ Session: aaa333 │         │ Session: bbb444 │        │
│ └─────────────────┘         └─────────────────┘        │
│                                                          │
│ Charlie (UID 1002)                                      │
│ ┌─────────────────┐                                     │
│ │ SSH Session 1   │                                     │
│ │ PID 30001       │                                     │
│ │ Session: ccc555 │                                     │
│ └─────────────────┘                                     │
└─────────────────────────────────────────────────────────┘

Daemon Perspective:
╔════════╦═══════════╦════════════╦═════════╦══════════════╗
║ User   ║ SessionID ║ Shell PID  ║ Status  ║ Independent? ║
╠════════╬═══════════╬════════════╬═════════╬══════════════╣
║ Alice  ║ aaa111    ║ 10001      ║ Active  ║ ✅ Yes       ║
║ Alice  ║ aaa333    ║ 10002      ║ Active  ║ ✅ Yes       ║
║ Bob    ║ bbb222    ║ 20001      ║ Active  ║ ✅ Yes       ║
║ Bob    ║ bbb444    ║ 20002      ║ Active  ║ ✅ Yes       ║
║ Charlie║ ccc555    ║ 30001      ║ Active  ║ ✅ Yes       ║
╚════════╩═══════════╩════════════╩═════════╩══════════════╝

Key Points:
• Each user has their own daemon instance (per-user service)
• Alice's daemon: /run/user/1000/terminal-lock.sock
• Bob's daemon:   /run/user/1001/terminal-lock.sock
• Charlie's daemon: /run/user/1002/terminal-lock.sock
• Users cannot interfere with each other's sessions
• System lock only affects the user who locked (if GUI session)
• Server reboot invalidates all sessions
```

---

## 🔒 Security Model

### Authentication Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User Opens New Terminal                                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Shell Hook Executes: terminal-lock check                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. CLI → IPC Request: ValidateSession { shell_pid: $$ }    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Daemon Checks Database                                   │
│    • Session exists for this PID?                           │
│    • Session not expired? (< 15 min old)                    │
│    • Session not idle? (< 5 min since last activity)        │
│    • Lock generation matches?                               │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         ▼                       ▼
    [Valid]                 [Invalid]
         │                       │
         ▼                       ▼
┌──────────────┐      ┌─────────────────────┐
│ 5a. Return   │      │ 5b. Show Auth       │
│ SessionValid │      │     Prompt          │
│              │      │                     │
│ Shell Ready  │      │ Password: ******    │
└──────────────┘      └─────────┬───────────┘
                                │
                                ▼
                      ┌─────────────────────┐
                      │ 6. Verify Password  │
                      │    (Argon2 hash)    │
                      └─────────┬───────────┘
                                │
                    ┌───────────┴───────────┐
                    ▼                       ▼
               [Correct]               [Incorrect]
                    │                       │
                    ▼                       ▼
          ┌──────────────────┐    ┌─────────────────┐
          │ 7a. Create       │    │ 7b. Increment   │
          │ New Session      │    │ Failed Attempts │
          │                  │    │                 │
          │ INSERT INTO      │    │ 3 attempts →    │
          │ sessions...      │    │ 60s lockout     │
          │                  │    │                 │
          │ Shell Ready      │    │ 9 total →       │
          └──────────────────┘    │ Lock system     │
                                  └─────────────────┘
```

### Rate Limiting

**Per-Terminal Limits**:
- 3 failed attempts → 60-second lockout
- After lockout expires → Reset counter, try again

**Global Limits**:
- 9 failed attempts total → Lock system (Super+L equivalent)
- Counter resets after successful system unlock

**Implementation**:
```sql
-- After 3 failed attempts in one terminal:
UPDATE users 
SET failed_attempts = 3, 
    locked_until = current_timestamp + 60 
WHERE uid = ?;

-- After 9 total failed attempts:
-- Trigger system lock via:
-- Linux: loginctl lock-session
-- macOS: pmset displaysleepnow
-- Windows: rundll32.exe user32.dll,LockWorkStation
```

### Password Storage

**Algorithm**: Argon2id (memory-hard, resistant to GPU attacks)

```rust
// Password hashing (on setup)
let config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 65536,      // 64 MB
    time_cost: 3,         // 3 iterations
    lanes: 4,             // 4 parallel lanes
    secret: &[],
    ad: &[],
    hash_length: 32,
};

let salt = generate_random_salt(16); // 16 bytes random
let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config)?;

// Store in database:
INSERT INTO users (uid, password_hash, salt) 
VALUES (?, ?, ?);
```

### Session Tokens

**Format**: UUID v4 (128-bit random)
```
a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

**Storage**: SQLite database with 0600 permissions
```bash
# Linux/macOS
~/.local/share/terminal-lock/sessions.db (rw-------)

# Windows
%APPDATA%\terminal-lock\sessions.db (User: Full Control, Others: None)
```

### Audit Logging

Every security event is logged:

```sql
-- Successful authentication
INSERT INTO audit_log (timestamp, event_type, user_id, shell_pid, details)
VALUES (1732622400, 'AUTH_SUCCESS', 1000, 12345, '{"ip": "192.168.1.100"}');

-- Failed authentication
INSERT INTO audit_log (timestamp, event_type, user_id, details)
VALUES (1732622401, 'AUTH_FAIL', 1000, '{"reason": "incorrect_password", "attempts": 1}');

-- System lock
INSERT INTO audit_log (timestamp, event_type, user_id, details)
VALUES (1732622500, 'SYSTEM_LOCK', 1000, '{"sessions_invalidated": 3}');

-- Session expiry
INSERT INTO audit_log (timestamp, event_type, user_id, session_id, details)
VALUES (1732623000, 'SESSION_EXPIRED', 1000, 'abc123', '{"reason": "hard_timeout"}');
```

**Log Rotation**: Automatic cleanup after 90 days (configurable)

---

## ⚙️ Configuration

### User Preferences File

**Location**: `~/.config/terminal-lock/preferences.json`

```json
{
  "ui": {
    "color_scheme": "vibrant",  // vibrant | pastel | minimal
    "animations": true,
    "animation_type": "matrix", // matrix | starfield | particles
    "show_greeting": true,
    "greeting_style": "dynamic" // dynamic | formal | casual
  },
  "security": {
    "session_timeout_minutes": 15,
    "idle_timeout_minutes": 5,
    "max_attempts_per_session": 3,
    "max_total_attempts": 9,
    "lockout_duration_seconds": 60,
    "require_auth_on_new_terminal": true,