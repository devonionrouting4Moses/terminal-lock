use colored::*;
use console::Term;
use terminal_lock_auth::{lockout::*, password::PasswordHasher, SystemLocker};
use terminal_lock_config::load_preferences;
use terminal_lock_core::{constants::*, error::*, Paths};
use terminal_lock_ui::{ColorPalette, LockoutDisplay};
use crate::ipc_helper;

pub async fn run(paths: &Paths) -> Result<()> {
    let term = Term::stderr();
    let prefs = load_preferences(paths)?;
    let palette = ColorPalette::new(prefs.color_scheme);

    // Check laptop unlock status first (resets counter)
    AttemptsTracker::check_laptop_unlock(paths)?;

    // Main authentication loop
    loop {
        // Check if locked out
        if let Some(until) = LockoutManager::is_locked(paths)? {
            // Show countdown (blocks until expires)
            LockoutDisplay::show_countdown(paths, until, &palette)?;
            // After lockout, continue to prompt again
            continue;
        }

        // Load current total attempts
        let mut total_attempts = AttemptsTracker::load(paths)?.total;

        // Check if already at max - lock system
        if total_attempts >= MAX_TOTAL_ATTEMPTS {
            eprintln!(
                "\n{}",
                "🚨 SECURITY ALERT: 9 failed attempts detected!".red().bold()
            );
            eprintln!("{}", "Locking your laptop for security...".red());
            std::thread::sleep(std::time::Duration::from_secs(2));
            SystemLocker::lock_system()?;
            
            // Wait for laptop to be unlocked, then reset counter in same terminal
            eprintln!("\n{}", "🔒 SYSTEM LOCKED".red().bold());
            eprintln!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red());
            eprintln!("\n{}", "⏳ Waiting for laptop unlock...".yellow().bold());
            eprintln!("{}", "This terminal will remain open and responsive.".cyan());
            eprintln!("{}", "Please unlock your laptop to continue.\n".cyan());
            std::io::Write::flush(&mut std::io::stderr()).ok();
            
            let mut wait_count = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                wait_count += 1;
                
                // Print status every 10 seconds
                if wait_count % 10 == 0 {
                    eprintln!("{}", format!("⏳ Still waiting... ({} seconds elapsed)", wait_count).yellow());
                    std::io::Write::flush(&mut std::io::stderr()).ok();
                }
                
                // Check if laptop was unlocked (marker file removed by system unlock)
                if AttemptsTracker::check_laptop_unlock(paths)? {
                    eprintln!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
                    eprintln!("{}", "✓ Laptop unlocked! Counter reset to 0/9".green().bold());
                    eprintln!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
                    eprintln!("{}", "Please try again with correct password.\n".cyan());
                    std::io::Write::flush(&mut std::io::stderr()).ok();
                    // Break out of wait loop and go back to main loop
                    break;
                }
            }
            // Continue to main loop to prompt for password again
            continue;
        }

        // Try up to 3 attempts before lockout
        let mut session_attempts = 0;

        while session_attempts < MAX_ATTEMPTS_PER_SESSION {
            // Show password prompt
            print!(
                "{}",
                palette.colorize("🔐 Enter terminal password:").bold()
            );
            std::io::Write::flush(&mut std::io::stdout()).ok();

            // Read password
            let password = term
                .read_secure_line()
                .map_err(|e| Error::Unknown(format!("Failed to read password: {}", e)))?;

            // Verify password
            if PasswordHasher::verify_stored_password(paths, &password)? {
                // SUCCESS!
                println!("\n{}", "✓ Authentication successful!".green().bold());

                // Reset attempts and create session
                AttemptsTracker::reset(paths)?;
                LockoutManager::cleanup_all_lockouts(paths)?;
                
                // Get shell PID from environment variable
                let shell_pid = std::env::var("TERMINAL_LOCK_SESSION_PID")
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or_else(|| std::process::id());
                
                // Get password hash for daemon
                let password_hash = PasswordHasher::get_stored_hash(paths)?;
                
                // Create session via IPC with fallback to file-based
                match ipc_helper::create_session_with_fallback(shell_pid, password_hash.trim(), paths) {
                    Ok(session_id) => {
                        eprintln!("✓ Session created: {} [{}]", 
                            session_id,
                            if session_id == "file-based" { "file-based" } else { "daemon" }
                        );
                    }
                    Err(e) => {
                        eprintln!("ERROR: Failed to create session: {}", e);
                        return Err(e);
                    }
                }

                return Ok(());
            }

            // FAILED ATTEMPT
            session_attempts += 1;
            total_attempts = AttemptsTracker::increment(paths)?;

            // Check if we hit 9 total attempts
            if total_attempts >= MAX_TOTAL_ATTEMPTS {
                eprintln!(
                    "\n{}",
                    "🚨 SECURITY ALERT: 9 failed attempts detected!".red().bold()
                );
                eprintln!("{}", "Locking your laptop for security...".red());
                std::thread::sleep(std::time::Duration::from_secs(2));
                SystemLocker::lock_system()?;
                
                // Wait for laptop to be unlocked, then reset counter in same terminal
                eprintln!("\n{}", "🔒 SYSTEM LOCKED".red().bold());
                eprintln!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red());
                eprintln!("\n{}", "⏳ Waiting for laptop unlock...".yellow().bold());
                eprintln!("{}", "This terminal will remain open and responsive.".cyan());
                eprintln!("{}", "Please unlock your laptop to continue.\n".cyan());
                std::io::Write::flush(&mut std::io::stderr()).ok();
                
                let mut wait_count = 0;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    wait_count += 1;
                    
                    // Print status every 10 seconds
                    if wait_count % 10 == 0 {
                        eprintln!("{}", format!("⏳ Still waiting... ({} seconds elapsed)", wait_count).yellow());
                        std::io::Write::flush(&mut std::io::stderr()).ok();
                    }
                    
                    // Check if laptop was unlocked (marker file removed by system unlock)
                    if AttemptsTracker::check_laptop_unlock(paths)? {
                        eprintln!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
                        eprintln!("{}", "✓ Laptop unlocked! Counter reset to 0/9".green().bold());
                        eprintln!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".green());
                        eprintln!("{}", "Please try again with correct password.\n".cyan());
                        std::io::Write::flush(&mut std::io::stderr()).ok();
                        // Break out of wait loop and go back to main loop
                        break;
                    }
                }
                // Continue to main loop to prompt for password again
                break;
            }

            // Still have attempts left in this cycle
            if session_attempts < MAX_ATTEMPTS_PER_SESSION {
                let remaining_in_cycle = MAX_ATTEMPTS_PER_SESSION - session_attempts;
                eprintln!(
                    "{}",
                    format!(
                        "✗ Invalid password. Attempt {} of {}",
                        session_attempts, MAX_ATTEMPTS_PER_SESSION
                    )
                    .red()
                    .bold()
                );
                eprintln!(
                    "{}",
                    format!(
                        "⚠ WARNING: {} more attempt(s) before 60s lockout!",
                        remaining_in_cycle
                    )
                    .yellow()
                );
                eprintln!(
                    "{}\n",
                    format!("Global attempts: {}/{}", total_attempts, MAX_TOTAL_ATTEMPTS).yellow()
                );
                // Continue to next prompt
                continue;
            }
        }

        // Hit 3 attempts in this cycle - create lockout
        let lockout_until = LockoutManager::create_lockout(paths)?;
        
        eprintln!(
            "\n{}",
            "⚠️  3 failed attempts in this cycle!".red().bold()
        );
        eprintln!(
            "{}",
            "🔒 Terminal locked for 60 seconds".red()
        );
        eprintln!(
            "{}\n",
            format!("📊 Total failed attempts: {}/{}", total_attempts, MAX_TOTAL_ATTEMPTS).yellow()
        );

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Show countdown
        LockoutDisplay::show_countdown(paths, lockout_until, &palette)?;

        // After countdown, loop back to prompt again
    }
}

// Helper function to show attempt status
fn show_attempt_status(session_attempt: u32, total_attempts: u32) {
    let remaining_in_cycle = MAX_ATTEMPTS_PER_SESSION - session_attempt;
    
    eprintln!(
        "{}",
        format!(
            "✗ Invalid password. Attempt {} of {} in this cycle",
            session_attempt, MAX_ATTEMPTS_PER_SESSION
        )
        .red()
        .bold()
    );
    
    if remaining_in_cycle > 0 {
        eprintln!(
            "{}",
            format!(
                "⚠ WARNING: {} more attempt(s) before 60s lockout!",
                remaining_in_cycle
            )
            .yellow()
        );
    }
    
    eprintln!(
        "{}\n",
        format!("📊 Total attempts: {}/{}", total_attempts, MAX_TOTAL_ATTEMPTS).yellow()
    );
}