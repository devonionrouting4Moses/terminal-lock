use console::Term;
use terminal_lock_core::error::{Error, Result};

/// Secure password input handler
pub struct PasswordPrompt;

impl PasswordPrompt {
    /// Prompt for password (hidden input)
    pub fn prompt(message: &str) -> Result<String> {
        let term = Term::stderr();
        
        // Show prompt
        term.write_line(message)
            .map_err(|e| Error::Unknown(format!("Failed to write prompt: {}", e)))?;

        // Read password securely (hidden)
        let password = term
            .read_secure_line()
            .map_err(|e| Error::Unknown(format!("Failed to read password: {}", e)))?;

        Ok(password)
    }

    /// Prompt for password confirmation
    pub fn prompt_with_confirmation(message: &str) -> Result<String> {
        loop {
            let password = Self::prompt(message)?;
            let confirm = Self::prompt("Confirm password: ")?;

            if password == confirm {
                return Ok(password);
            }

            eprintln!("Passwords don't match. Please try again.\n");
        }
    }

    /// Read a line of input (visible)
    pub fn read_line(message: &str) -> Result<String> {
        let term = Term::stderr();
        
        term.write_line(message)
            .map_err(|e| Error::Unknown(format!("Failed to write prompt: {}", e)))?;

        let input = term
            .read_line()
            .map_err(|e| Error::Unknown(format!("Failed to read input: {}", e)))?;

        Ok(input.trim().to_string())
    }

    /// Show a menu and get selection
    pub fn show_menu(title: &str, options: &[&str]) -> Result<usize> {
        let term = Term::stderr();

        loop {
            term.clear_screen()
                .map_err(|e| Error::Unknown(format!("Failed to clear screen: {}", e)))?;

            term.write_line(title)
                .map_err(|e| Error::Unknown(format!("Failed to write title: {}", e)))?;
            term.write_line("")
                .map_err(|e| Error::Unknown(format!("Failed to write newline: {}", e)))?;

            for (i, option) in options.iter().enumerate() {
                term.write_line(&format!("  {}) {}", i + 1, option))
                    .map_err(|e| Error::Unknown(format!("Failed to write option: {}", e)))?;
            }

            term.write_line("")
                .map_err(|e| Error::Unknown(format!("Failed to write newline: {}", e)))?;

            let input = Self::read_line(&format!("Choose (1-{}): ", options.len()))?;

            if let Ok(choice) = input.parse::<usize>() {
                if choice >= 1 && choice <= options.len() {
                    return Ok(choice - 1);
                }
            }

            term.write_line("Invalid choice. Please try again.")
                .map_err(|e| Error::Unknown(format!("Failed to write error: {}", e)))?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    /// Ask yes/no question
    pub fn confirm(message: &str, default: bool) -> Result<bool> {
        let default_str = if default { "[Y/n]" } else { "[y/N]" };
        let prompt = format!("{} {}: ", message, default_str);

        let input = Self::read_line(&prompt)?;

        if input.is_empty() {
            return Ok(default);
        }

        Ok(matches!(input.to_lowercase().as_str(), "y" | "yes"))
    }
}