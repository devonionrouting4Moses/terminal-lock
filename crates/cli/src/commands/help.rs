use colored::*;
use terminal_lock_core::error::Result;
use terminal_lock_ui::Table;
use terminal_lock_core::types::ColorScheme;

pub fn run() -> Result<()> {
    // Header with description
    let table = Table::new(terminal_lock_ui::ColorPalette::new(ColorScheme::Vibrant))
        .with_title("Terminal Lock")
        .with_headers(vec!["Feature"])
        .add_row(vec!["ℹ Enhanced terminal authentication system with animations"])
        .add_row(vec!["🔐 Protects your terminal with session-based security"])
        .render();
    
    println!("{}", table);
    println!();

    // Commands table
    let table = Table::new(terminal_lock_ui::ColorPalette::new(ColorScheme::Vibrant))
        .with_title("Available Commands")
        .with_headers(vec!["Command", "Description"])
        .add_row(vec!["setup", "Initial setup - configure password and preferences"])
        .add_row(vec!["preferences", "Manage preferences (color scheme, animations, etc.)"])
        .add_row(vec!["reset", "Reset attempts counter and lockouts"])
        .add_row(vec!["status", "Check authentication status and session info"])
        .add_row(vec!["check", "Check if session is valid (blocks terminal if invalid)"])
        .add_row(vec!["lock", "Lock terminal manually"])
        .add_row(vec!["extend", "Extend session (update activity timestamp)"])
        .add_row(vec!["daemon", "Start lock monitor daemon (monitors system lock events)"])
        .add_row(vec!["auth", "Run authentication (default when called from shell)"])
        .add_row(vec!["help", "Print this help message"])
        .render();
    
    println!("{}", table);
    println!();

    // Options table
    let table = Table::new(terminal_lock_ui::ColorPalette::new(ColorScheme::Vibrant))
        .with_title("Global Options")
        .with_headers(vec!["Option", "Description"])
        .add_row(vec!["-h, --help", "Print help message"])
        .add_row(vec!["-V, --version", "Print version information"])
        .render();
    
    println!("{}", table);
    println!();

    // Examples table
    let table = Table::new(terminal_lock_ui::ColorPalette::new(ColorScheme::Vibrant))
        .with_title("Usage Examples")
        .with_headers(vec!["Command", "Description"])
        .add_row(vec!["terminal-lock setup", "Run initial setup wizard"])
        .add_row(vec!["terminal-lock status", "Display current session status"])
        .add_row(vec!["terminal-lock auth", "Authenticate to create a session"])
        .add_row(vec!["terminal-lock lock", "Manually lock the terminal"])
        .add_row(vec!["terminal-lock preferences", "Modify user preferences"])
        .render();
    
    println!("{}", table);
    println!();

    // Info box
    let info_table = Table::new(terminal_lock_ui::ColorPalette::new(ColorScheme::Vibrant))
        .with_title("Need More Help?")
        .with_headers(vec!["Info"])
        .add_row(vec!["ℹ For more information, run: terminal-lock <command> --help"])
        .add_row(vec!["🔒 Session-based security for your terminal"])
        .render();
    
    println!("{}", info_table);
    println!();

    Ok(())
}
