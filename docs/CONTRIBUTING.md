# Contributing to Terminal Lock

Thank you for considering contributing to Terminal Lock! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the issue, not the person
- Help others learn and grow

## How to Contribute

### Reporting Bugs

Before creating a bug report:
1. Check existing issues
2. Verify it's reproducible
3. Test with latest version

Bug report should include:
- Terminal Lock version (`terminal-lock --version`)
- Operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

### Suggesting Features

Feature requests should include:
- Clear use case
- Expected behavior
- Alternative solutions considered
- Willingness to implement

### Submitting Pull Requests

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/terminal-lock
   cd terminal-lock
   ```

2. **Create Branch**
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

3. **Make Changes**
   - Follow coding standards (see below)
   - Write tests
   - Update documentation

4. **Test**
   ```bash
   make ci  # Runs fmt, clippy, and tests
   ```

5. **Commit**
   ```bash
   git commit -m "feat: add awesome feature"
   ```
   Follow [Conventional Commits](https://www.conventionalcommits.org/)

6. **Push**
   ```bash
   git push origin feature/my-awesome-feature
   ```

7. **Open Pull Request**
   - Describe changes clearly
   - Reference related issues
   - Add screenshots if UI changes

## Development Setup

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Git

### Initial Setup

```bash
# Clone repository
git clone https://github.com/yourusername/terminal-lock
cd terminal-lock

# Build
cargo build

# Run tests
cargo test --workspace

# Run specific example
cargo run --example color_demo
```

### Useful Commands

```bash
# Run all checks (format, lint, test)
make ci

# Format code
make fmt

# Run clippy
make clippy

# Run tests
make test

# Run specific test
cargo test -p terminal-lock-auth test_password_validation

# Run benchmarks
make bench

# Generate docs
make docs
```

## Coding Standards

### Rust Style

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// ✅ Good
pub fn create_session(paths: &Paths) -> Result<()> {
    // Implementation
}

// ❌ Bad - no error handling
pub fn create_session(paths: &Paths) {
    // Implementation
}
```

### Formatting

Use `rustfmt`:

```bash
cargo fmt --all
```

Configuration in `rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
```

### Linting

Pass `clippy` with no warnings:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Documentation

Document public APIs:

```rust
/// Creates a new session with default timeout.
///
/// # Arguments
///
/// * `paths` - File path configuration
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if file operations fail.
///
/// # Examples
///
/// ```
/// use terminal_lock_core::Paths;
/// use terminal_lock_core::session::Session;
///
/// let paths = Paths::new()?;
/// Session::create(&paths)?;
/// ```
pub fn create_session(paths: &Paths) -> Result<()> {
    // Implementation
}
```

### Error Handling

Use custom error types:

```rust
// ✅ Good
pub fn authenticate(&self) -> Result<()> {
    let password = self.prompt_password()?;
    if !self.verify(password)? {
        return Err(Error::AuthenticationFailed("Invalid password".into()));
    }
    Ok(())
}

// ❌ Bad - using anyhow for library
pub fn authenticate(&self) -> anyhow::Result<()> {
    // ...
}
```

### Testing

Write tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        assert!(PasswordValidator::validate("Test123").is_ok());
        assert!(PasswordValidator::validate("test123").is_err());
    }

    #[test]
    fn test_session_creation() {
        let paths = setup_test_paths();
        assert!(Session::create(&paths).is_ok());
    }
}
```

Integration tests in `tests/` directory:

```rust
// tests/auth_tests.rs
use terminal_lock_auth::Authenticator;

#[test]
fn test_full_auth_flow() {
    // Test complete authentication workflow
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for YubiKey authentication
fix: resolve session timeout race condition
docs: update security documentation
test: add integration tests for lockout
refactor: simplify password validation logic
perf: optimize color palette generation
chore: update dependencies
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

## Project Structure

```
terminal-lock/
├── crates/
│   ├── core/          # Add core types here
│   ├── auth/          # Add authentication logic here
│   ├── config/        # Add configuration logic here
│   ├── quotes/        # Add quote APIs here
│   ├── ui/            # Add UI components here
│   └── cli/           # Add CLI commands here
├── examples/          # Add usage examples here
├── benches/           # Add benchmarks here
├── tests/             # Add integration tests here
└── docs/              # Add documentation here
```

## Adding Features

### Adding a New Quote API

1. Add to `crates/quotes/src/apis.rs`:
   ```rust
   pub fn get_quote_apis() -> Vec<QuoteApi> {
       vec![
           // ... existing APIs
           QuoteApi::new("New API", "https://api.example.com/quote", QuoteSource::General),
       ]
   }
   ```

2. Implement parser if needed:
   ```rust
   pub fn parse_response(api_name: &str, body: &str) -> Option<Quote> {
       if api_name.contains("New API") {
           // Custom parsing logic
       }
       // ...
   }
   ```

3. Test the API:
   ```bash
   cargo run --example quote_fetcher
   ```

### Adding a New Color Scheme

1. Add to `crates/core/src/types.rs`:
   ```rust
   pub enum ColorScheme {
       Vibrant,
       Pastel,
       Minimal,
       NewScheme,  // Add here
   }
   ```

2. Implement in `crates/ui/src/colors.rs`:
   ```rust
   impl ColorPalette {
       fn new_scheme_color(&self, rng: &mut impl Rng) -> ColoredString {
           // Implementation
       }
   }
   ```

3. Update setup wizard in `crates/cli/src/commands/setup.rs`

### Adding a New Command

1. Create file in `crates/cli/src/commands/`:
   ```rust
   // my_command.rs
   use terminal_lock_core::{error::Result, Paths};

   pub async fn run(paths: &Paths) -> Result<()> {
       // Implementation
       Ok(())
   }
   ```

2. Add to `crates/cli/src/commands/mod.rs`:
   ```rust
   pub mod my_command;
   ```

3. Add to main CLI in `crates/cli/src/main.rs`:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // ... existing commands
       MyCommand,
   }

   match cli.command {
       // ... existing matches
       Some(Commands::MyCommand) => {
           commands::my_command::run(&paths).await?;
       }
   }
   ```

## Documentation

### Code Documentation

Use rustdoc comments:

```rust
/// Brief description
///
/// More detailed explanation...
///
/// # Arguments
///
/// * `arg1` - Description
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns errors
///
/// # Examples
///
/// ```
/// // Example usage
/// ```
pub fn function(arg1: Type) -> Result<ReturnType> {
    // ...
}
```

### README Updates

Update `README.md` for:
- New features
- Changed behavior
- New dependencies
- Installation changes

### Changelog

Update `CHANGELOG.md` (if exists) for each release.

## Review Process

### What Reviewers Look For

1. **Correctness**
   - Does it work as intended?
   - Are there edge cases?

2. **Tests**
   - Adequate test coverage?
   - Tests actually test the feature?

3. **Documentation**
   - Public APIs documented?
   - README updated if needed?

4. **Style**
   - Follows Rust conventions?
   - Passes `rustfmt` and `clippy`?

5. **Performance**
   - No obvious performance issues?
   - Benchmarks if performance-critical?

6. **Security**
   - No security vulnerabilities?
   - Follows security best practices?

### Getting Reviews

- Tag specific reviewers if needed
- Respond to feedback promptly
- Update PR based on feedback
- Mark conversations as resolved

## Release Process

(For maintainers)

1. **Version Bump**
   ```bash
   # Update version in all Cargo.toml files
   # Update CHANGELOG.md
   ```

2. **Test**
   ```bash
   make ci
   cargo test --workspace --release
   ```

3. **Tag**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

4. **Publish**
   ```bash
   cargo publish -p terminal-lock-core
   cargo publish -p terminal-lock-auth
   # ... etc
   cargo publish -p terminal-lock-cli
   ```

## Getting Help

- **Questions**: Open a discussion on GitHub
- **Bugs**: Open an issue with bug report template
- **Security**: Email security@yourproject.com
- **Chat**: Join our Discord/Slack (if available)

## Recognition

Contributors will be:
- Listed in AUTHORS file
- Mentioned in release notes
- Credited in commit history

Thank you for contributing! 🚀