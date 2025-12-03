use colored::*;
use chrono::Utc;
use terminal_lock_auth::lockout::{AttemptsTracker, LockoutManager};
use terminal_lock_config::{load_preferences, DisplayNameManager};
use terminal_lock_core::{constants::*, error::Result, session::Session, Paths};
use crate::ipc_helper;
// use std::time::Duration;

pub fn run(paths: &Paths) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════════════════╗".blue().bold());
    println!("{}", "║        Terminal Lock - Status Report                  ║".blue().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════╝".blue().bold());
    println!();

    // Configuration status
    println!("{}", "Configuration:".cyan().bold());
    
    if paths.pass_hash_file().exists() {
        println!("  {} Password configured", "✓".green());
    } else {
        println!("  {} Password NOT configured (run setup)", "✗".red());
        return Ok(());
    }

    let name = DisplayNameManager::load(paths)?;
    println!("  {} Display name: {}", "✓".green(), name.bold());

    if let Ok(prefs) = load_preferences(paths) {
        println!("  {} Color scheme: {}", "✓".green(), prefs.color_scheme);
        println!("  {} Animations: {}", "✓".green(), prefs.animations);
        println!("  {} Quote source: {}", "✓".green(), prefs.quote_source);
        println!("  {} Greeting style: {}", "✓".green(), prefs.greeting_style);
    }
    println!();

    // Daemon status
    println!("{}", "Daemon Status:".cyan().bold());
    if ipc_helper::is_daemon_running() {
        println!("  {} Daemon is RUNNING", "✓".green());
        println!("  {} IPC: Enabled", "✓".green());
    } else {
        println!("  {} Daemon is NOT running", "○".yellow());
        println!("  {} IPC: Disabled (using file-based fallback)", "ℹ".cyan());
    }
    println!();

    // Session status
    println!("{}", "Session Status:".cyan().bold());
    if Session::is_valid(paths)? {
        println!("  {} Session is ACTIVE", "✓".green());
        println!("  {} Timeout: {} minutes", "ℹ".cyan(), SESSION_TTL.as_secs() / 60);
    } else {
        println!("  {} No active session", "○".yellow());
        println!("  {} Authentication required", "!".yellow());
    }
    println!();

    // Lockout status
    println!("{}", "Security Status:".cyan().bold());
    
    if let Some(until) = LockoutManager::is_locked(paths)? {
        let now = Utc::now();
        let remaining = (until - now).to_std().unwrap_or_default().as_secs() as i64;
        println!(
            "  {} Terminal is LOCKED ({} seconds remaining)",
            "🔒".red(),
            remaining
        );
    } else {
        println!("  {} Terminal is unlocked", "✓".green());
    }

    let attempts_info = AttemptsTracker::load(paths)?;
    let total = attempts_info.total;

    if total == 0 {
        println!("  {} No failed attempts", "✓".green());
    } else {
        let status = if total >= MAX_TOTAL_ATTEMPTS / 2 {
            format!("{}/{}", total, MAX_TOTAL_ATTEMPTS).red()
        } else {
            format!("{}/{}", total, MAX_TOTAL_ATTEMPTS).yellow()
        };
        println!("  {} Failed attempts: {}", "⚠".yellow(), status);
        
        if total >= MAX_TOTAL_ATTEMPTS {
            println!(
                "  {} CRITICAL: Maximum attempts reached! Laptop will lock on next failure.",
                "🚨".red()
            );
        }
    }
    println!();

    // Limits
    println!("{}", "Limits:".cyan().bold());
    println!(
        "  • Per-terminal attempts: {} (lockout: {}s)",
        MAX_ATTEMPTS_PER_SESSION, LOCKOUT_DURATION.as_secs()
    );
    println!("  • Global max attempts: {} (laptop locks)", MAX_TOTAL_ATTEMPTS);
    println!("  • Session timeout: {} minutes", SESSION_TTL.as_secs() / 60);
    println!();

    Ok(())
}