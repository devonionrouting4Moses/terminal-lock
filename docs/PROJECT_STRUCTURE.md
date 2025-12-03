# Terminal Lock - Complete Project Structure

This document outlines all files needed for the complete project.

## Directory Tree

```
terminal-lock/
├── Cargo.toml                    # Workspace root
├── Makefile                      # Build automation
├── install.sh                    # Universal installer script
├── README.md                     # Project documentation
├── LICENSE                       # MIT License
├── .gitignore                    # Git ignore rules
├── .github/
│   └── workflows/
│       ├── ci.yml                # CI/CD pipeline
│       └── release.yml           # Release automation
│
├── crates/
│   ├── core/                     # Core types and utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs          # Error types
│   │       ├── types.rs          # Core types (Preferences, Quote, etc.)
│   │       ├── paths.rs          # File path management
│   │       └── session.rs        # Session management
│   │
│   ├── auth/                     # Authentication logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── password.rs       # Password hashing & validation
│   │       ├── lockout.rs        # Lockout & attempts tracking
│   │       ├── system_lock.rs    # System locking
│   │       └── authenticator.rs  # Main authentication logic
│   │
│   ├── config/                   # Configuration management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── manager.rs        # Configuration manager
│   │       └── display_name.rs   # Display name handling
│   │
│   ├── quotes/                   # Quote fetching
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── apis.rs           # Quote API definitions (43+ APIs)
│   │       └── fetcher.rs        # Async quote fetching
│   │
│   ├── ui/                       # Terminal UI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── colors.rs         # Color palette (1000+ colors)
│   │       ├── greeting.rs       # Greeting display
│   │       ├── input.rs          # Password input
│   │       ├── lockout.rs        # Lockout countdown UI
│   │       └── animations.rs     # Background animations
│   │
│   └── cli/                      # CLI application
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # Entry point
│           ├── utils.rs          # Utility functions
│           └── commands/
│               ├── mod.rs
│               ├── auth.rs       # Authentication command
│               ├── greeting.rs   # Greeting display
│               ├── lock.rs       # Manual lock
│               ├── preferences.rs # Preferences manager
│               ├── reset.rs      # Reset attempts
│               ├── setup.rs      # Initial setup wizard
│               └── status.rs     # Status check
│
├── examples/                     # Usage examples
│   ├── custom_auth.rs            # Custom authentication
│   ├── quote_fetcher.rs          # Quote fetching example
│   └── color_demo.rs             # Color palette demo
│
├── benches/                      # Benchmarks
│   ├── auth_bench.rs             # Authentication benchmarks
│   ├── password_bench.rs         # Password hashing benchmarks
│   └── quote_bench.rs            # Quote fetching benchmarks
│
├── tests/                        # Integration tests
│   ├── auth_tests.rs
│   ├── config_tests.rs
│   └── session_tests.rs
│
└── docs/                         # Additional documentation
    ├── ARCHITECTURE.md           # Architecture overview
    ├── API.md                    # API documentation
    ├── SECURITY.md               # Security details
    └── CONTRIBUTING.md           # Contribution guidelines
```

## File Contents Summary

### Root Files

1. **Cargo.toml** - Workspace configuration with all crates
2. **Makefile** - Build, test, install targets
3. **install.sh** - Shell script to build and install
4. **README.md** - Complete project documentation
5. **.gitignore** - Ignore target/, Cargo.lock (for libraries)

### Core Crate (`crates/core/`)

Contains fundamental types and error handling:
- **error.rs**: Custom Error enum with thiserror
- **types.rs**: Preferences, Quote, Session, Lockout types
- **paths.rs**: File path management (~/.terminal_lock/)
- **session.rs**: Session validation and management
- **lib.rs**: Re-exports and constants

### Auth Crate (`crates/auth/`)

Handles all authentication logic:
- **password.rs**: Argon2 hashing, validation, storage
- **lockout.rs**: Lockout state, attempts tracking
- **system_lock.rs**: System lock commands (loginctl, etc.)
- **authenticator.rs**: Main authentication orchestration

### Config Crate (`crates/config/`)

Manages user preferences:
- **manager.rs**: CRUD operations for preferences
- **display_name.rs**: Display name storage

### Quotes Crate (`crates/quotes/`)

Fetches motivational quotes:
- **apis.rs**: 43+ API definitions with categories
- **fetcher.rs**: Async HTTP requests with timeout

### UI Crate (`crates/ui/`)

Terminal display and animations:
- **colors.rs**: 1000+ color palette generation
- **greeting.rs**: Dynamic greeting display
- **input.rs**: Secure password input
- **lockout.rs**: Countdown display during lockout
- **animations.rs**: Background animations (matrix, starfield)

### CLI Crate (`crates/cli/`)

Main application:
- **main.rs**: CLI parsing with clap, subcommands
- **commands/**: Individual command implementations
  - setup: Initial wizard
  - auth: Authentication flow
  - preferences: Interactive preferences
  - status: Show current status
  - reset: Reset counters

## Build Order

```bash
# 1. Create directory structure
mkdir -p crates/{core,auth,config,quotes,ui,cli}/src
mkdir -p examples benches tests docs

# 2. Create all Cargo.toml files
# (Root workspace + each crate)

# 3. Implement crates in dependency order:
#    core -> auth, config, quotes, ui -> cli

# 4. Build
cargo build --release

# 5. Install
make install
```

## Key Features by Crate

### Core
- ✅ Type-safe enums (ColorScheme, QuoteSource, GreetingStyle)
- ✅ Error handling with thiserror
- ✅ Session management with timeout
- ✅ File path abstractions

### Auth
- ✅ Argon2 password hashing
- ✅ Per-terminal lockout (3 attempts)
- ✅ Global attempt tracking (9 max)
- ✅ System lock integration
- ✅ Laptop unlock detection

### Config
- ✅ JSON-based preferences
- ✅ Import/export
- ✅ Default values
- ✅ Type-safe updates

### Quotes
- ✅ 43+ working APIs
- ✅ Category filtering
- ✅ Timeout handling (3s)
- ✅ Fallback quotes
- ✅ Async/blocking modes

### UI
- ✅ 1000+ dynamic colors
- ✅ 3 color schemes (Vibrant/Pastel/Minimal)
- ✅ Typewriter effect
- ✅ Lockout countdown
- ✅ Background animations

### CLI
- ✅ Subcommand-based CLI
- ✅ Interactive setup wizard
- ✅ Preferences TUI
- ✅ Shell integration
- ✅ Status reporting

## Testing Strategy

```bash
# Unit tests (in each crate)
cargo test -p terminal-lock-core
cargo test -p terminal-lock-auth

# Integration tests
cargo test --test auth_tests

# All tests
cargo test --workspace

# Benchmarks
cargo bench
```

## Installation Methods

### Method 1: From Source
```bash
git clone https://github.com/yourusername/terminal-lock
cd terminal-lock
make install
```

### Method 2: Installer Script
```bash
curl -sSL https://raw.githubusercontent.com/.../install.sh | bash
```

### Method 3: Cargo Install
```bash
cargo install terminal-lock-cli
```

## Configuration Files

All stored in `~/.terminal_lock/`:

```
~/.terminal_lock/
├── pass.hash              # Argon2 password hash
├── salt                   # Random salt (16 chars)
├── display_name           # User's display name
├── preferences.json       # User preferences
├── session.<pid>.json     # Active sessions (per terminal)
├── lockout.<pid>.json     # Lockout states (per terminal)
├── total_attempts.json    # Global attempts counter
└── last_unlock            # Last laptop unlock timestamp
```

## Shell Integration

### Bash (~/.bashrc)
```bash
if [[ $- == *i* ]]; then
    terminal-lock auth 2>/dev/null || true
fi
```

### Zsh (~/.zshrc)
```bash
if [[ -o interactive ]]; then
    terminal-lock auth 2>/dev/null || true
fi
```

### Fish (~/.config/fish/config.fish)
```fish
if status is-interactive
    terminal-lock auth
end
```

## Release Checklist

- [ ] All tests passing
- [ ] Clippy warnings resolved
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created
- [ ] Binary artifacts built
- [ ] Release notes written

## Dependencies

### Core Dependencies
- serde, serde_json (serialization)
- anyhow, thiserror (error handling)
- chrono (timestamps)
- dirs (home directory)

### Crypto
- argon2 (password hashing)
- rand (randomness)

### Terminal UI
- crossterm (terminal control)
- colored (color output)
- console (terminal features)

### HTTP
- reqwest (HTTP client)
- tokio (async runtime)

### CLI
- clap (argument parsing)

## Performance Targets

- Startup: < 50ms
- Authentication: < 100ms
- Quote fetch: < 3s (with timeout)
- Memory usage: < 10MB
- Binary size: < 5MB (release)

## Security Features

- ✅ Argon2id password hashing
- ✅ Random salt per password
- ✅ Secure file permissions (600)
- ✅ Session timeout (15 min)
- ✅ Progressive lockout
- ✅ System lock after 9 attempts
- ✅ No plaintext passwords
- ✅ Constant-time comparison