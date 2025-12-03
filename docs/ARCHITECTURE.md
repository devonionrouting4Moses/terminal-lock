# Terminal Lock - Architecture

## Overview

Terminal Lock is a modular, secure terminal authentication system built with Rust. The architecture follows a layered approach with clear separation of concerns.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     CLI Layer                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Commands: setup, auth, preferences, status, etc │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  Application Layer                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────┐  │
│  │  Auth   │  │ Config  │  │ Quotes  │  │    UI    │  │
│  └─────────┘  └─────────┘  └─────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                    Core Layer                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Types, Errors, Paths, Sessions, Constants       │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                External Dependencies                    │
│  Argon2, Tokio, Crossterm, Reqwest, Serde, etc.        │
└─────────────────────────────────────────────────────────┘
```

## Crate Responsibilities

### Core (`terminal-lock-core`)
**Purpose**: Foundation types and utilities

**Responsibilities**:
- Define core types (Preferences, Quote, Session, etc.)
- Error handling with thiserror
- File path management
- Session validation
- Global constants

**Key Files**:
- `error.rs` - Error types and Result alias
- `types.rs` - Core data structures
- `paths.rs` - File path abstractions
- `session.rs` - Session management logic

**No Dependencies On**: Other terminal-lock crates (leaf crate)

### Auth (`terminal-lock-auth`)
**Purpose**: Authentication and security

**Responsibilities**:
- Password hashing with Argon2
- Password validation
- Lockout management
- Attempts tracking
- System locking
- Main authentication orchestration

**Key Files**:
- `password.rs` - Hashing, validation, storage
- `lockout.rs` - Lockout state and attempts
- `system_lock.rs` - OS-level locking
- `authenticator.rs` - Authentication flow

**Dependencies**: `core`

### Config (`terminal-lock-config`)
**Purpose**: Configuration management

**Responsibilities**:
- Load/save preferences
- Display name management
- Configuration CRUD operations
- Import/export preferences

**Key Files**:
- `manager.rs` - Config manager
- `display_name.rs` - Name handling

**Dependencies**: `core`

### Quotes (`terminal-lock-quotes`)
**Purpose**: Motivational quote fetching

**Responsibilities**:
- Define 43+ quote APIs
- Async HTTP requests
- Timeout handling
- Fallback quotes
- Category filtering

**Key Files**:
- `apis.rs` - API definitions and parsing
- `fetcher.rs` - Async/blocking fetching

**Dependencies**: `core`

### UI (`terminal-lock-ui`)
**Purpose**: Terminal user interface

**Responsibilities**:
- Color palette generation (1000+ colors)
- Greeting display with animations
- Password input
- Lockout countdown UI
- Background animations

**Key Files**:
- `colors.rs` - Color generation
- `greeting.rs` - Greeting display
- `input.rs` - Secure input
- `lockout.rs` - Lockout UI
- `animations.rs` - Background effects

**Dependencies**: `core`

### CLI (`terminal-lock-cli`)
**Purpose**: Main application binary

**Responsibilities**:
- Command-line interface
- Subcommand routing
- Shell integration
- Orchestrate all components

**Key Files**:
- `main.rs` - Entry point
- `commands/*.rs` - Command implementations

**Dependencies**: All other crates

## Data Flow

### Authentication Flow

```
User opens terminal
       │
       ▼
Check if session valid (Session::is_valid)
       │
       ├─ Valid ──────────────► Show greeting & exit
       │
       └─ Invalid
           │
           ▼
       Check lockout (LockoutManager::is_locked)
           │
           ├─ Locked ─────────► Show countdown & wait
           │
           └─ Not locked
               │
               ▼
           Prompt for password
               │
               ▼
           Hash & verify (PasswordHasher)
               │
               ├─ Success ────► Create session & show greeting
               │
               └─ Failure
                   │
                   ├─ < 3 attempts ──► Retry
                   │
                   ├─ 3 attempts ────► Lock terminal (60s)
                   │
                   └─ 9 total ───────► Lock system
```

### Quote Fetching Flow

```
Request quote with category
       │
       ▼
Filter APIs by category
       │
       ▼
Select random API
       │
       ▼
HTTP request with 3s timeout
       │
       ├─ Success ──────► Parse response ──► Return quote
       │
       └─ Timeout/Error ──────────────────► Return fallback
```

### Session Management Flow

```
User authenticates
       │
       ▼
Create session file with expiry (15 min)
       │
       ▼
On each terminal use:
   Check session file
       │
       ├─ Expired ───────► Delete file & require auth
       │
       └─ Valid ─────────► Extend expiry & continue
```

## Security Model

### Password Storage
1. User enters password
2. Generate random 16-char salt
3. Hash with Argon2id (memory-hard)
4. Store hash and salt separately (600 permissions)
5. Never store plaintext

### Session Security
- Session files use process ID (per-terminal)
- 15-minute timeout (configurable)
- Extends on each validation check
- Removed on logout/expiry

### Lockout Protection
- **Per-terminal**: 3 attempts → 60s lockout
- **Global**: 9 attempts → laptop lock
- Resets on laptop unlock detection
- Prevents brute-force attacks

## File System Layout

```
~/.terminal_lock/
├── pass.hash                    # Argon2 password hash
├── salt                         # Random salt
├── display_name                 # User's name
├── preferences.json             # User preferences
├── session.<pid>.json           # Active sessions
├── lockout.<pid>.json           # Lockout states
├── total_attempts.json          # Global counter
└── last_unlock                  # Unlock timestamp
```

All files have 600 permissions (owner read/write only).

## Concurrency Model

### Async Operations
- Quote fetching (Tokio)
- HTTP requests (Reqwest)
- Parallel API calls

### Synchronous Operations
- Password hashing (blocking)
- File I/O (simple)
- Session checks

### Thread Safety
- Each terminal has separate session file (PID-based)
- Lockout files are per-terminal
- Global counter uses file locking (implicit)

## Error Handling Strategy

### Error Types
```rust
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    AuthenticationFailed(String),
    SessionExpired,
    TerminalLocked(DateTime),
    SystemLocked,
    ConfigNotFound,
    // ...
}
```

### Error Propagation
1. Low-level errors (IO, JSON) → wrapped in custom Error
2. Business logic errors → specific variants
3. All functions return `Result<T, Error>`
4. CLI layer handles and displays errors

## Performance Characteristics

### Startup Time
- **Target**: < 50ms
- **Measured**: ~30-40ms typical
- **Bottlenecks**: None significant

### Authentication Time
- **Target**: < 100ms
- **Measured**: ~80-90ms (includes Argon2)
- **Bottleneck**: Password hashing (intentional)

### Quote Fetching
- **Target**: < 3s
- **Measured**: Variable (network dependent)
- **Timeout**: 3s hard limit
- **Fallback**: Always available

### Memory Usage
- **Target**: < 10MB
- **Measured**: ~5-8MB typical
- **Profile**: Minimal allocations

## Extensibility Points

### Adding New Quote APIs
1. Add URL to `apis.rs`
2. Implement parser if needed
3. Categorize by `QuoteSource`

### Adding New Color Schemes
1. Add variant to `ColorScheme` enum
2. Implement in `ColorPalette`
3. Update UI prompts

### Adding New Greeting Styles
1. Add variant to `GreetingStyle` enum
2. Implement in `GreetingDisplay`
3. Update preferences

### Custom Authentication
```rust
let authenticator = Authenticator::new(paths);
authenticator.authenticate(|| {
    // Custom password prompt
    my_custom_prompt()
});
```

## Testing Strategy

### Unit Tests
- Each module has `#[cfg(test)]` section
- Test pure functions in isolation
- Mock external dependencies

### Integration Tests
- Tests in `tests/` directory
- Use `tempfile` for isolated file systems
- Test full workflows end-to-end

### Benchmarks
- Criterion for performance tests
- Track password hashing time
- Monitor quote fetch latency

## Build and Release

### Debug Build
```bash
cargo build
# Fast compilation, debug symbols
```

### Release Build
```bash
cargo build --release
# Optimizations: LTO, single codegen unit
```

### Binary Size
- Debug: ~15-20MB
- Release: ~4-5MB (stripped)

## Dependencies Graph

```
┌────────────────────────────────────────────┐
│                    CLI                     │
│  (clap, tokio, tracing, console)          │
└────────────────────────────────────────────┘
          │       │       │       │
    ┌─────┘       │       │       └─────┐
    │             │       │             │
    ▼             ▼       ▼             ▼
┌──────┐    ┌──────┐ ┌──────┐      ┌─────┐
│ Auth │    │Config│ │Quotes│      │ UI  │
└──────┘    └──────┘ └──────┘      └─────┘
    │             │       │             │
    └─────┐       │       │       ┌─────┘
          │       │       │       │
          ▼       ▼       ▼       ▼
      ┌────────────────────────────┐
      │           Core             │
      │  (serde, chrono, dirs)    │
      └────────────────────────────┘
```

## Future Enhancements

### Planned Features
- [ ] Biometric authentication support
- [ ] Web dashboard for remote management
- [ ] Cloud sync for preferences
- [ ] Multi-user support
- [ ] Plugin system for extensions
- [ ] Mobile app for remote unlock

### Architecture Changes
- [ ] Consider gRPC for remote features
- [ ] Add database for audit logs
- [ ] Implement webhook notifications
- [ ] Add telemetry (opt-in)