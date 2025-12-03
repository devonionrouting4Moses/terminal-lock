# Terminal Lock - API Documentation

## CLI API

### Commands

#### `terminal-lock setup`

Initial configuration wizard.

**Usage:**
```bash
terminal-lock setup
```

**Interactive Prompts:**
1. Display name
2. Password (6 chars, 1 upper, 1 lower, 1 digit)
3. Color scheme (Vibrant/Pastel/Minimal)
4. Animations (yes/no)
5. Quote source category
6. Greeting style

**Example:**
```bash
$ terminal-lock setup
╔═══════════════════════════════════════════════════════╗
║   Terminal Lock Setup - Enhanced with Animations     ║
╚═══════════════════════════════════════════════════════╝

Step 1: Display Name
How would you like to be greeted? DevMoses
✓ Display name set to: DevMoses

Step 2: Password Setup
...
```

#### `terminal-lock auth`

Authenticate user (default command).

**Usage:**
```bash
terminal-lock auth
```

**Behavior:**
- Checks if session is valid
- If valid: Shows greeting and exits
- If invalid: Prompts for password
- Blocks terminal until authentication succeeds

**Exit Codes:**
- `0` - Authentication successful
- `1` - Authentication failed/system locked

#### `terminal-lock status`

Show current authentication status.

**Usage:**
```bash
terminal-lock status
```

**Output:**
```
╔═══════════════════════════════════════════════════════╗
║        Terminal Lock - Status Report                  ║
╚═══════════════════════════════════════════════════════╝

Configuration:
  ✓ Password configured
  ✓ Display name: DevMoses
  ✓ Color scheme: vibrant
  ✓ Animations: true
  ✓ Quote source: all
  ✓ Greeting style: dynamic

Session Status:
  ✓ Session is ACTIVE
  ℹ Timeout: 15 minutes

Security Status:
  ✓ Terminal is unlocked
  ✓ No failed attempts

Limits:
  • Per-terminal attempts: 3 (lockout: 60s)
  • Global max attempts: 9 (laptop locks)
  • Session timeout: 15 minutes
```

#### `terminal-lock preferences`

Interactive preference manager.

**Usage:**
```bash
terminal-lock preferences
```

**Options:**
1. Change Display Name
2. Change Color Scheme
3. Toggle Animations
4. Change Quote Source
5. Change Greeting Style
6. Reset to Defaults
7. Export Preferences
8. Import Preferences
9. Exit

#### `terminal-lock reset`

Reset attempts counter and clear lockouts.

**Usage:**
```bash
terminal-lock reset
```

**Output:**
```
Resetting Terminal Lock state...

✓ Attempts counter reset to 0
✓ All lockouts cleared
✓ All sessions cleared

Reset complete! You can now authenticate again.
```

#### `terminal-lock lock`

Manually lock the current terminal.

**Usage:**
```bash
terminal-lock lock
```

**Effect:**
- Removes current session
- Next command requires authentication

## Library API

### Core Module (`terminal-lock-core`)

#### `Paths`

File path management.

```rust
use terminal_lock_core::Paths;

let paths = Paths::new()?;
paths.ensure_lock_dir()?;

// Access file paths
paths.pass_hash_file();
paths.session_file();
paths.preferences_file();
```

#### `Session`

Session management.

```rust
use terminal_lock_core::session::Session;

// Check if valid
if Session::is_valid(&paths)? {
    println!("Session active");
}

// Create new session
Session::create(&paths)?;

// Remove session
Session::remove(&paths)?;
```

#### Types

```rust
use terminal_lock_core::types::{
    ColorScheme, QuoteSource, GreetingStyle, 
    Preferences, Quote
};

// Color schemes
let scheme = ColorScheme::Vibrant;

// Quote sources
let source = QuoteSource::Coding;

// Preferences
let prefs = Preferences {
    color_scheme: ColorScheme::Vibrant,
    animations: true,
    quote_source: QuoteSource::All,
    greeting_style: GreetingStyle::Dynamic,
};
```

### Auth Module (`terminal-lock-auth`)

#### `PasswordHasher`

Password hashing and verification.

```rust
use terminal_lock_auth::PasswordHasher;

// Generate salt
let salt = PasswordHasher::generate_salt();

// Hash password
let hash = PasswordHasher::hash_password("Pass123", &salt)?;

// Verify password
let valid = PasswordHasher::verify_password("Pass123", &hash)?;

// Save password to disk
PasswordHasher::save_password(&paths, "Pass123")?;

// Verify against stored password
let valid = PasswordHasher::verify_stored_password(&paths, "Pass123")?;
```

#### `PasswordValidator`

Password format validation.

```rust
use terminal_lock_auth::PasswordValidator;

// Validate format
match PasswordValidator::validate("Pass123") {
    Ok(_) => println!("Valid password"),
    Err(e) => eprintln!("Invalid: {}", e),
}
```

#### `Authenticator`

Main authentication orchestrator.

```rust
use terminal_lock_auth::Authenticator;

let authenticator = Authenticator::new(paths);

// Check if authentication needed
if authenticator.needs_authentication()? {
    // Authenticate with custom prompt
    authenticator.authenticate(|| {
        // Your custom password prompt
        let password = read_password()?;
        Ok(password)
    })?;
}

// Get attempt counts
let (total, max) = authenticator.get_attempts()?;
println!("Attempts: {}/{}", total, max);
```

#### `LockoutManager`

Lockout state management.

```rust
use terminal_lock_auth::LockoutManager;

// Check if locked
if let Some(until) = LockoutManager::is_locked(&paths)? {
    println!("Locked until: {}", until);
}

// Create lockout
let until = LockoutManager::create_lockout(&paths)?;

// Remove lockout
LockoutManager::remove_lockout(&paths)?;

// Cleanup all lockouts
LockoutManager::cleanup_all_lockouts(&paths)?;
```

### Config Module (`terminal-lock-config`)

#### `ConfigManager`

Configuration management.

```rust
use terminal_lock_config::ConfigManager;

let mut manager = ConfigManager::new(paths)?;

// Get preferences
let prefs = manager.preferences();

// Update settings
manager.set_color_scheme(ColorScheme::Pastel)?;
manager.set_animations(false)?;
manager.set_quote_source(QuoteSource::Coding)?;
manager.set_greeting_style(GreetingStyle::Formal)?;

// Reset to defaults
manager.reset_to_defaults()?;

// Export/Import
manager.export_to_file(&path)?;
manager.import_from_file(&path)?;
```

#### `DisplayNameManager`

Display name management.

```rust
use terminal_lock_config::DisplayNameManager;

// Load name
let name = DisplayNameManager::load(&paths)?;

// Save name
DisplayNameManager::save(&paths, "DevMoses")?;
```

### Quotes Module (`terminal-lock-quotes`)

#### `QuoteFetcher`

Quote fetching from APIs.

```rust
use terminal_lock_quotes::QuoteFetcher;
use terminal_lock_core::types::QuoteSource;

let fetcher = QuoteFetcher::new();

// Async fetch
let quote = fetcher.fetch(QuoteSource::All).await;

// Blocking fetch
let quote = fetcher.fetch_blocking(QuoteSource::Coding);

println!("\"{}\"", quote.text);
if let Some(author) = quote.author {
    println!("  — {}", author);
}
```

### UI Module (`terminal-lock-ui`)

#### `ColorPalette`

Color generation.

```rust
use terminal_lock_ui::ColorPalette;
use terminal_lock_core::types::ColorScheme;

let palette = ColorPalette::new(ColorScheme::Vibrant);

// Get random color
let color = palette.random_color();
println!("{}", color);

// Colorize text
let text = palette.colorize("Hello World");
println!("{}", text);

// Border
let border = palette.border();
```

#### `PasswordPrompt`

Secure input.

```rust
use terminal_lock_ui::PasswordPrompt;

// Password input (hidden)
let password = PasswordPrompt::prompt("Enter password: ")?;

// With confirmation
let password = PasswordPrompt::prompt_with_confirmation("Enter password: ")?;

// Visible input
let name = PasswordPrompt::read_line("Enter name: ")?;

// Menu
let choice = PasswordPrompt::show_menu(
    "Choose an option",
    &["Option 1", "Option 2", "Option 3"],
)?;

// Confirmation
let yes = PasswordPrompt::confirm("Continue?", true)?;
```

#### `GreetingDisplay`

Greeting generation and display.

```rust
use terminal_lock_ui::GreetingDisplay;
use terminal_lock_core::types::GreetingStyle;

// Generate greeting
let greeting = GreetingDisplay::generate_greeting("DevMoses", GreetingStyle::Dynamic);

// Display with quote
GreetingDisplay::display("DevMoses", GreetingStyle::Dynamic, quote, &palette);

// Show loading animation
GreetingDisplay::show_loading("Fetching inspiration...");
```

## Error Handling

All functions return `Result<T, Error>`:

```rust
use terminal_lock_core::error::{Error, Result};

fn my_function() -> Result<()> {
    let paths = Paths::new()?;
    
    match some_operation() {
        Ok(value) => Ok(value),
        Err(Error::SessionExpired) => {
            // Handle expired session
        }
        Err(Error::TerminalLocked(until)) => {
            // Handle lockout
        }
        Err(e) => Err(e),
    }
}
```

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
    InvalidPasswordFormat(String),
    PasswordMismatch,
    InvalidPreference(String),
    QuoteApiError(String),
    Unknown(String),
}
```

## File Formats

### Preferences JSON

```json
{
  "color_scheme": "vibrant",
  "animations": true,
  "quote_source": "all",
  "greeting_style": "dynamic"
}
```

### Session JSON

```json
{
  "expiry": "2025-01-15T10:30:00Z",
  "terminal_id": "12345"
}
```

### Lockout JSON

```json
{
  "until": "2025-01-15T10:05:00Z",
  "terminal_id": "12345"
}
```

### Attempts JSON

```json
{
  "total": 2,
  "last_boot_time": 1705312800
}
```

## Environment Variables

### `RUST_LOG`

Enable logging:

```bash
RUST_LOG=debug terminal-lock auth
RUST_LOG=info terminal-lock status
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Authentication failed / Error |

## Shell Integration

### Zsh

```bash
if [[ -o interactive ]]; then
    if command -v terminal-lock &> /dev/null; then
        terminal-lock auth 2>/dev/null
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi
fi
```

### Bash

```bash
if [[ $- == *i* ]]; then
    if command -v terminal-lock &> /dev/null; then
        terminal-lock auth 2>/dev/null
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi
fi
```

## Examples

See `examples/` directory for complete examples:

- `custom_auth.rs` - Custom authentication
- `quote_fetcher.rs` - Quote fetching
- `color_demo.rs` - Color palette demo