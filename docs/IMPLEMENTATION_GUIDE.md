# Terminal Lock - Quick Reference Card

## 🚀 Installation

```bash
# Build and install
make install

# Or use installer
chmod +x install.sh && ./install.sh

# Add to shell (~/.bashrc or ~/.zshrc)
if [[ $- == *i* ]]; then terminal-lock auth; fi
```

## 📋 Commands

| Command | Description |
|---------|-------------|
| `terminal-lock setup` | Initial setup wizard |
| `terminal-lock auth` | Authenticate (default) |
| `terminal-lock preferences` | Manage preferences |
| `terminal-lock status` | Show current status |
| `terminal-lock reset` | Reset attempts/lockouts |
| `terminal-lock lock` | Lock terminal manually |

## 🔑 Password Requirements

- ✅ Exactly **6 characters**
- ✅ **1+ uppercase** letter (A-Z)
- ✅ **1+ lowercase** letter (a-z)
- ✅ **1+ digit** (0-9)

Examples: `Test123`, `Pass1A`, `Code42X`

## 🔒 Security Limits

| Limit | Value | Action |
|-------|-------|--------|
| **Per-terminal** | 3 attempts | 60s lockout |
| **Global** | 9 attempts | Laptop locks |
| **Session** | 15 minutes | Auto-logout |

## 🎨 Color Schemes

- **Vibrant**: 1000+ dynamic colors
- **Pastel**: Soft, easy on eyes
- **Minimal**: 4 simple colors

## 💬 Quote Sources

- **All**: 43 APIs, max variety
- **Coding**: Tech & programming
- **Spiritual**: Zen & wisdom
- **Motivational**: Success & inspiration
- **General**: Famous quotes

## 🖥️ Greeting Styles

| Style | Example |
|-------|---------|
| **Dynamic** | "Good morning, Achiever Moses!" |
| **Formal** | "Good morning, Moses." |
| **Casual** | "Hey Moses! Ready to code?" |

## 📁 File Locations

```
~/.terminal_lock/
├── pass.hash          # Password hash
├── salt               # Random salt
├── display_name       # Your name
├── preferences.json   # Settings
├── session.<pid>      # Active sessions
└── total_attempts     # Global counter
```

## 🛠️ Development

```bash
# Build
cargo build

# Test
cargo test --workspace

# Format
cargo fmt --all

# Lint
cargo clippy --workspace

# Run example
cargo run --example color_demo

# Benchmark
cargo bench
```

## 🐛 Troubleshooting

```bash
# Reset everything
terminal-lock reset

# Check status
terminal-lock status

# Enable debug
RUST_LOG=debug terminal-lock auth

# Clean slate
rm -rf ~/.terminal_lock && terminal-lock setup
```

## ⚡ Performance

- **Startup**: ~30ms
- **Auth**: ~80ms
- **Memory**: <10MB
- **Binary**: 5MB

## 🔧 Configuration

```bash
# Change preferences interactively
terminal-lock preferences

# Export settings
terminal-lock preferences  # Choose "Export"

# Import settings
terminal-lock preferences  # Choose "Import"
```

## 📊 Status Codes

| Status | Meaning |
|--------|---------|
| ✓ | Success / Active |
| ○ | Inactive |
| ⚠ | Warning |
| ✗ | Error |
| 🔒 | Locked |

## 🎯 Quick Tasks

### Change Password
```bash
rm ~/.terminal_lock/pass.hash ~/.terminal_lock/salt
terminal-lock setup
```

### Change Display Name
```bash
terminal-lock preferences  # Choose "Change Display Name"
```

### Clear Lockout
```bash
terminal-lock reset
```

### Test Quote APIs
```bash
cargo run --example quote_fetcher
```

### View All Colors
```bash
cargo run --example color_demo
```

## 📦 Crate Structure

```
terminal-lock/
├── core      # Types, errors, paths
├── auth      # Authentication logic
├── config    # Preferences management
├── quotes    # Quote fetching
├── ui        # Terminal UI
└── cli       # Main application
```

## 🔐 Security Best Practices

1. ✅ Use strong password
2. ✅ Lock laptop when away
3. ✅ Don't share credentials
4. ✅ Monitor attempts regularly
5. ✅ Keep system updated

## 📚 Documentation

- **README.md**: Overview & usage
- **ARCHITECTURE.md**: System design
- **SECURITY.md**: Security details
- **CONTRIBUTING.md**: How to contribute
- **FILE_CHECKLIST.md**: Implementation guide

## 🆘 Getting Help

```bash
# Check version
terminal-lock --version

# View help
terminal-lock --help

# Command help
terminal-lock setup --help
```

## 🎓 Learning Path

1. **Start**: Run `terminal-lock setup`
2. **Use**: Authenticate daily
3. **Customize**: Try different preferences
4. **Explore**: Run examples
5. **Extend**: Add new features!

---

**Keep this card handy for quick reference!** 📌