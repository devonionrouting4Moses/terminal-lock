use colored::*;
use terminal_lock_auth::lockout::{AttemptsTracker, LockoutManager};
use terminal_lock_core::{error::Result, session::SessionManager, Paths};

pub fn run(paths: &Paths) -> Result<()> {
    println!("{}", "Resetting Terminal Lock state...".cyan().bold());
    println!();

    // Reset attempts counter
    AttemptsTracker::reset(paths)?;
    println!("{}", "✓ Attempts counter reset to 0".green());

    // Remove all lockouts
    LockoutManager::cleanup_all_lockouts(paths)?;
    println!("{}", "✓ All lockouts cleared".green());

    // Remove all sessions
    SessionManager::remove_all_sessions(paths)?;
    println!("{}", "✓ All sessions cleared".green());

    println!();
    println!("{}", "Reset complete! You can now authenticate again.".green().bold());

    Ok(())
}