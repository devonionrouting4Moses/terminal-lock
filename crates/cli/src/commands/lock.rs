use colored::*;
use terminal_lock_core::{error::Result, session::Session, Paths};

pub fn run(paths: &Paths) -> Result<()> {
    println!("{}", "Locking terminal...".yellow().bold());

    // Remove current session
    Session::remove(paths)?;

    println!("{}", "✓ Terminal locked successfully".green());
    println!();
    println!(
        "{}",
        "You will need to authenticate again to use this terminal.".cyan()
    );

    Ok(())
}