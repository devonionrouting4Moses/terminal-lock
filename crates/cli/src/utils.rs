/// Check if running in an interactive shell
pub fn is_interactive_shell() -> bool {
    // Check if stdin is a TTY
    atty::is(atty::Stream::Stdin)
}

/// Get terminal dimensions
pub fn get_terminal_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}

/// Check if terminal supports colors
pub fn supports_color() -> bool {
    atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size() {
        let (cols, rows) = get_terminal_size();
        assert!(cols > 0);
        assert!(rows > 0);
    }
}