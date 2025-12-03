// crates/ui/src/lockout.rs
// FINAL FIX: Single updating countdown box with proper screen clearing

use chrono::{DateTime, Utc};
use colored::*;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use terminal_lock_auth::lockout::AttemptsTracker;
use terminal_lock_core::{constants::MAX_TOTAL_ATTEMPTS, error::Result, Paths};

use super::ColorPalette;

/// Lockout countdown display
pub struct LockoutDisplay;

impl LockoutDisplay {
    /// Show lockout countdown (blocks terminal completely)
    /// FIXED: Single updating box, no scrolling
    pub fn show_countdown(
        paths: &Paths,
        until: DateTime<Utc>,
        palette: &ColorPalette,
    ) -> Result<()> {
        // Use stdout instead of stderr for better control
        let mut stdout = io::stdout();
        
        // Clear screen once at the beginning
        print!("\x1B[2J\x1B[H");
        stdout.flush().ok();

        loop {
            let now = Utc::now();
            let remaining = (until - now).num_seconds();

            if remaining <= 0 {
                break;
            }

            // Move cursor to home position (top-left corner)
            print!("\x1B[H");

            // Get total attempts
            let total = AttemptsTracker::load(paths)?.total;

            // Build the box content
            Self::draw_box(&mut stdout, remaining, total, palette)?;

            // Flush immediately
            stdout.flush()?;

            // Wait 1 second
            thread::sleep(Duration::from_secs(1));
        }

        // Clear screen after countdown
        print!("\x1B[2J\x1B[H");
        println!(
            "\n{}",
            "⏳ Lockout period expired. You may try again...".cyan().bold()
        );
        thread::sleep(Duration::from_secs(2));
        print!("\x1B[2J\x1B[H");
        stdout.flush()?;

        Ok(())
    }

    /// Draw the lockout box (rewritten to avoid println! during countdown)
    fn draw_box(
        stdout: &mut io::Stdout,
        remaining: i64,
        total: u32,
        palette: &ColorPalette,
    ) -> Result<()> {
        let border = "═".repeat(59);
        let empty = " ".repeat(59);

        // Build entire box as raw strings (no println!)
        let lines = vec![
            palette.colorize(&format!("╔{}╗", border)).bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize("║           🚫 TERMINAL LOCKED - ACCESS DENIED             ║").bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize("║  3 failed password attempts detected!                    ║").bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize(&format!(
                "║  Lockout Time Remaining: {:2} seconds{}║",
                remaining,
                " ".repeat(23)
            )).bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize("║  This terminal is completely BLOCKED.                    ║").bold().to_string(),
            palette.colorize("║  You cannot use it until lockout expires.                ║").bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize("║  After lockout: You will get 3 more attempts             ║").bold().to_string(),
            palette.colorize(&format!(
                "║  Total failed attempts so far: {}/{}{}║",
                total,
                MAX_TOTAL_ATTEMPTS,
                " ".repeat(27)
            )).bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize("║  ⚠️  Warning: After 9 total failed attempts,             ║").bold().to_string(),
            palette.colorize("║      your laptop will be locked automatically!           ║").bold().to_string(),
            palette.colorize(&format!("║{}║", empty)).bold().to_string(),
            palette.colorize(&format!("╚{}╝", border)).bold().to_string(),
        ];

        // Write all lines at once
        for line in lines {
            write!(stdout, "{}\n", line)?;
        }

        Ok(())
    }

    /// Show warning after failed attempt
    pub fn show_attempt_warning(
        attempt: u32,
        max_attempts: u32,
        total: u32,
        palette: &ColorPalette,
    ) {
        let remaining = max_attempts - attempt;

        println!(
            "{}",
            format!(
                "✗ Invalid password. Attempt {} of {}",
                attempt, max_attempts
            )
            .red()
            .bold()
        );

        if remaining > 0 {
            println!(
                "{}",
                format!(
                    "⚠ WARNING: {} more attempt(s) before THIS terminal locks for 60s!",
                    remaining
                )
                .yellow()
            );
            println!(
                "{}",
                format!("Global attempts: {}/{}\n", total, MAX_TOTAL_ATTEMPTS).yellow()
            );
        }
    }
}

// Alternative implementation using crossterm (more reliable)
#[cfg(feature = "crossterm")]
use crossterm::{
    cursor, execute,
    terminal::{self, Clear, ClearType},
};

#[cfg(feature = "crossterm")]
impl LockoutDisplay {
    /// Alternative implementation with crossterm
    pub fn show_countdown_crossterm(
        paths: &Paths,
        until: DateTime<Utc>,
        palette: &ColorPalette,
    ) -> Result<()> {
        let mut stdout = io::stdout();
        
        // Hide cursor
        execute!(stdout, cursor::Hide).ok();
        
        // Enable raw mode to prevent scrolling
        terminal::enable_raw_mode().ok();

        loop {
            let now = Utc::now();
            let remaining = (until - now).num_seconds();

            if remaining <= 0 {
                break;
            }

            // Clear screen and move to top
            execute!(
                stdout,
                Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            ).ok();

            // Get total attempts
            let total = AttemptsTracker::load(paths)?.total;

            // Draw box
            Self::draw_box(&mut stdout, remaining, total, palette)?;

            // Flush
            stdout.flush().ok();

            // Wait 1 second
            thread::sleep(Duration::from_secs(1));
        }

        // Cleanup
        terminal::disable_raw_mode().ok();
        execute!(stdout, cursor::Show).ok();

        // Clear and show message
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        ).ok();

        println!(
            "\n{}",
            "⏳ Lockout period expired. You may try again...".cyan().bold()
        );
        thread::sleep(Duration::from_secs(2));

        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        ).ok();

        Ok(())
    }
}