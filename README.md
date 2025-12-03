# Terminal Lock 🔐

A secure, feature-rich terminal authentication system written in Rust with beautiful animations and customizable preferences.

## Features ✨

- **🔒 Secure Authentication**: Argon2 password hashing with salt
- **⏱️ Per-Terminal Sessions**: Each terminal has independent 15-minute timeout
- **🚨 Smart Lockout**: 3 attempts per terminal, 9 total before system lock
- **🎨 1000+ Colors**: Vibrant, Pastel, or Minimal color schemes
- **🎭 Background Animations**: Matrix rain, starfield, particles
- **💬 43+ Quote APIs**: Motivational quotes from multiple sources
- **⚙️ Customizable**: Greeting styles, animations, quote categories
- **🔄 Independent Terminals**: Each terminal window requires its own authentication

## Architecture 🏗️

```
terminal-lock/
├── crates/
│   ├── core/          # Core types, errors, paths, session management
│   ├── auth/          # Authentication, password hashing, lockout logic
│   ├── config/        # Preferences and configuration management
│   ├── quotes/        # Quote fetching from 43+ APIs
│   ├── ui/            # Terminal UI, animations, colors
│   └── cli/           # Main CLI application
├── examples/          # Usage examples
├── benches/           # Performance benchmarks
└── README.md
```

## Installation 🚀

### From Source

```bash
# Clone repository
git clone https://github.com/yourusername/terminal-lock
cd terminal-lock

# Build release binary
cargo build --release

# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/terminal-lock ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$PATH:$HOME/.local/bin"
```

### Shell Integration

Add to your shell configuration:

**Bash** (`~/.bashrc`):
```bash
# Terminal Lock Authentication
if [[ $- == *i* ]]; then
    terminal-lock auth
fi
```

**Zsh** (`~/.zshrc`):
```bash
# Terminal Lock Authentication
if [[ -o interactive ]]; then
    terminal-lock auth 2>/dev/null || true
fi
```

**Fish** (`~/.config/fish/config.fish`):
```fish
# Terminal Lock Authentication
if status is-interactive
    terminal-lock auth
end
```

## Quick Start 🎯

### 1. Initial Setup

```bash
terminal-lock setup
```

You'll be prompted to:
- Set your display name (e.g., "Moses", "DevMoses")
- Create a 6-character password (1 uppercase, 1 lowercase, 1 digit)
- Choose color scheme (Vibrant/Pastel/Minimal)
- Enable/disable animations
- Select quote source category
- Choose greeting style

### 2. Using Terminal Lock

Open a new terminal - you'll be prompted for authentication:

```
═══════════════════════════════════════════════════════════
🔐 Enter terminal password: ******
✓ Authentication successful!
═══════════════════════════════════════════════════════════
Good morning, Achiever Moses!

"The best way to predict the future is to invent it."
  — Alan Kay
═══════════════════════════════════════════════════════════
```

### 3. Manage Preferences

```bash
terminal-lock preferences
```

Interactive menu to change:
- Display name
- Color scheme
- Animations on/off
- Quote categories
- Greeting style

## Commands 📋

```bash
terminal-lock setup          # Initial configuration
terminal-lock preferences    # Manage preferences
terminal-lock status         # Check authentication status
terminal-lock reset          # Reset attempts counter
terminal-lock lock           # Lock terminal manually
terminal-lock auth           # Run authentication (default)
```

## Security Features 🛡️

### Password Requirements
- Exactly 6 characters
- At least 1 uppercase letter (A-Z)
- At least 1 lowercase letter (a-z)
- At least 1 digit (0-9)

### Lockout System
- **Per-Terminal**: 3 failed attempts → 60-second lockout
- **Global**: 9 total failed attempts → laptop locks automatically
- **Auto-Reset**: Counter resets when laptop is unlocked
- **Session**: 15-minute timeout per terminal

### Encryption
- **Argon2**: Memory-hard password hashing
- **Random Salt**: 16-character salt per password
- **Secure Files**: 600 permissions on all config files

## Customization 🎨

### Color Schemes

**Vibrant** (1000+ colors)
- Dynamic RGB combinations
- Full 256-color palette
- Gradient variations

**Pastel** (Soft colors)
- Easy on eyes
- 16 curated pastel shades

**Minimal** (4 colors)
- Simple and clean
- Green, blue, cyan, magenta

### Quote Categories

- **All**: Maximum variety (43 APIs)
- **Coding/Tech**: Programming wisdom
- **Spiritual**: Zen, philosophy, wisdom
- **Motivational**: Success, inspiration
- **General**: Famous quotes, life advice

### Greeting Styles

**Dynamic**
- Morning: "Good morning, Achiever Moses!"
- Afternoon: "Good afternoon, Developer Moses!"
- Evening: "Good evening, Cloud Engineer Moses!"
- Night: "How's the night coffee with coding, System Engineer Moses?"

**Formal**
- "Good morning, Moses."

**Casual**
- "Hey Moses! Ready to code?"

## Development 🔧

### Project Structure

```
crates/
├── core/
│   ├── error.rs       # Error types
│   ├── types.rs       # Core types (Preferences, Quote, etc.)
│   ├── paths.rs       # File path management
│   └── session.rs     # Session management
│
├── auth/
│   ├── password.rs    # Password hashing & validation
│   ├── lockout.rs     # Lockout & attempts tracking
│   ├── system_lock.rs # System locking
│   └── authenticator.rs # Main authentication logic
│
├── config/
│   ├── manager.rs     # Configuration management
│   └── display_name.rs # Display name handling
│
├── quotes/
│   ├── apis.rs        # Quote API definitions
│   └── fetcher.rs     # Async quote fetching
│
├── ui/
│   ├── colors.rs      # Color palette generation
│   ├── greeting.rs    # Greeting display
│   ├── input.rs       # Password input
│   ├── lockout.rs     # Lockout countdown UI
│   └── animations.rs  # Background animations
│
└── cli/
    ├── main.rs        # CLI entry point
    ├── commands/      # Command implementations
    └── utils.rs       # Utilities
```

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run specific crate tests
cargo test -p terminal-lock-auth

# Run with logs
RUST_LOG=debug cargo run -- auth
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific module
cargo test -p terminal-lock-core

# With output
cargo test -- --nocapture
```

### Benchmarks

```bash
cargo bench
```

## Configuration Files 📁

All files stored in `~/.terminal_lock/`:

```
~/.terminal_lock/
├── pass.hash              # Password hash (Argon2)
├── salt                   # Random salt
├── display_name           # User's display name
├── preferences.json       # User preferences
├── session.<pid>.json     # Session files (per terminal)
├── lockout.<pid>.json     # Lockout files (per terminal)
├── total_attempts.json    # Global attempts counter
└── last_unlock            # Last laptop unlock time
```

## Troubleshooting 🔍

### "Configuration not found"
```bash
terminal-lock setup
```

### Reset everything
```bash
rm -rf ~/.terminal_lock
terminal-lock setup
```

### Clear lockouts
```bash
terminal-lock reset
```

### Check status
```bash
terminal-lock status
```

## Platform Support 🖥️

- ✅ Linux (Ubuntu, Debian, Arch, Fedora, etc.)
- ✅ macOS
- ⚠️ Windows (limited - WSL recommended)

## Performance 📊

- **Startup**: < 50ms
- **Authentication**: < 100ms
- **Quote Fetch**: < 3s (with timeout)
- **Memory**: < 10MB

## Contributing 🤝

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request

## License 📄

MIT License - see [LICENSE](LICENSE)

## Credits 👏

Built by Moses with ❤️ using Rust

Quote APIs:
- Quotable API
- ZenQuotes
- Advice Slip
- And 40+ more!

## Roadmap 🗺️

- [ ] Web dashboard for remote management
- [ ] Biometric authentication support
- [ ] Multi-user support
- [ ] Cloud sync preferences
- [ ] Plugin system
- [ ] Mobile app for remote unlock

---

**⭐ Star this project if you find it useful!**