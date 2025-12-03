use terminal_lock_core::{constants::*, error::*, paths::Paths, session::Session};

use super::{
    attempts::{AttemptsTracker, AttemptPhase, LockoutManager},
    password::PasswordHasher,
    system::SystemLocker,
};

/// Main authenticator
pub struct Authenticator {
    paths: Paths,
}

impl Authenticator {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    /// Check if authentication is needed
    pub fn needs_authentication(&self) -> Result<bool> {
        // Check if session is valid
        if Session::is_valid(&self.paths)? {
            return Ok(false);
        }

        // Check if password is configured
        if !self.paths.pass_hash_file().exists() {
            return Err(Error::ConfigNotFound);
        }

        Ok(true)
    }

    /// Authenticate user with password prompt
    /// Uses 9-attempt 3-phase system with countdown timers:
    /// - Phase 1 (1-3): Warning, show 30s countdown (color changes at 15s)
    /// - Phase 2 (4-6): Delay, show 30s countdown (color changes at 15s)
    /// - Phase 3 (7-9): Lock, system locks only after 9th attempt
    /// 
    /// After 9th attempt:
    /// - System locks entire machine
    /// - When user unlocks machine, attempts reset to 0/9
    /// - User starts fresh (second trial)
    /// TODO: Implement second trial behavior (future enhancement)
    pub fn authenticate<F>(&self, prompt_fn: F) -> Result<()>
    where
        F: Fn() -> Result<String>,
    {
        // Check laptop unlock status (resets counter if machine was locked)
        let was_locked = AttemptsTracker::check_laptop_unlock(&self.paths)?;
        
        // If machine was just unlocked after system lock, reset attempts to 0/9
        if was_locked {
            eprintln!("✅ Machine unlocked. Attempts reset to 0/9. Starting fresh trial...");
            // TODO: Implement second trial tracking (future enhancement)
            // - Track trial number (1st, 2nd, 3rd, etc.)
            // - Log trial history
            // - Implement escalating consequences for multiple trials
        }

        // Load current attempt info
        let (total_attempts, phase, _delay_secs, _message) = 
            AttemptsTracker::get_info(&self.paths)?;

        // Check if system is locked (only after MAX_TOTAL_ATTEMPTS in Phase 3)
        if phase == AttemptPhase::Lock && total_attempts >= MAX_TOTAL_ATTEMPTS {
            if let Some(remaining_secs) = LockoutManager::remaining_lockout_time(&self.paths)? {
                eprintln!(
                    "🔒 System locked. Try again in {} seconds",
                    remaining_secs
                );
                return Err(Error::SystemLocked);
            }
        }

        // Prompt for password
        let password = prompt_fn()?;

        if PasswordHasher::verify_stored_password(&self.paths, &password)? {
            // ✅ SUCCESS - Reset counters and create session
            eprintln!("✅ Authentication successful!");
            AttemptsTracker::reset(&self.paths)?;
            LockoutManager::cleanup_all_lockouts(&self.paths)?;
            Session::create(&self.paths)?;
            return Ok(());
        }

        // ❌ FAILED - Increment attempts and apply phase-based behavior
        let (new_total, new_phase, _new_delay, new_message) = 
            AttemptsTracker::increment(&self.paths)?;

        eprintln!("{}", new_message);

        // Apply phase-based behavior with strategic countdown timers
        match new_phase {
            AttemptPhase::Warning => {
                // Phase 1 (1-3): Countdown only on attempt 3
                if new_total == MAX_ATTEMPTS_PER_SESSION {
                    eprintln!("⏳ Final warning: 30-second countdown before lockout");
                    self.show_countdown_timer(30)?;
                }
                return Err(Error::AuthenticationFailed("Authentication failed".to_string()));
            }
            AttemptPhase::Delay => {
                // Phase 2 (4-6): Countdown only on attempt 6
                if new_total == (MAX_ATTEMPTS_PER_SESSION * 2) {
                    eprintln!("⏳ Critical: 30-second countdown before lockout");
                    self.show_countdown_timer(30)?;
                }
                return Err(Error::AuthenticationFailed("Authentication failed".to_string()));
            }
            AttemptPhase::Lock => {
                // Phase 3 (7-9): Check if we've reached MAX_TOTAL_ATTEMPTS
                if new_total >= MAX_TOTAL_ATTEMPTS {
                    // Lock system only after 9th attempt
                    eprintln!("🔒 System locked due to too many failed attempts!");
                    eprintln!("🔐 Locking entire machine...");
                    
                    // Mark system as locked (for reset on unlock)
                    AttemptsTracker::mark_system_locked(&self.paths)?;
                    
                    // Lock the entire machine
                    SystemLocker::lock_system().ok();
                    
                    // Create lockout using LOCKOUT_DURATION constant
                    let lockout_secs = LOCKOUT_DURATION.as_secs();
                    LockoutManager::create_lockout(&self.paths, lockout_secs, "Maximum attempts exceeded")?;
                    
                    eprintln!("🔒 Machine is now locked. Unlock to reset attempts to 0/9.");
                    return Err(Error::SystemLocked);
                } else {
                    // Attempts 7-8: Just show message, no countdown
                    return Err(Error::AuthenticationFailed("Authentication failed".to_string()));
                }
            }
        }
    }

    /// Show countdown timer with Cyberpunk Neon color scheme
    /// First half: Matrix green (#00ff41)
    /// Second half: Hot pink (#ff006e)
    fn show_countdown_timer(&self, total_seconds: u64) -> Result<()> {
        for remaining in (0..=total_seconds).rev() {
            // Cyberpunk Neon color scheme
            let color = if remaining > (total_seconds / 2) {
                // First half: Matrix green (safe zone)
                "\x1b[38;2;0;255;65m" // RGB: #00ff41 (matrix green)
            } else {
                // Second half: Hot pink (warning zone)
                "\x1b[38;2;255;0;110m" // RGB: #ff006e (hot pink)
            };
            let reset = "\x1b[0m";

            eprint!(
                "\r{}⏱️  Time remaining: {:02}:{:02}{} ",
                color,
                remaining / 60,
                remaining % 60,
                reset
            );
            std::io::Write::flush(&mut std::io::stderr()).ok();

            if remaining > 0 {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        eprintln!(); // New line after countdown
        Ok(())
    }

    /// Get current attempt info with phase
    pub fn get_attempts(&self) -> Result<(u32, AttemptPhase, String)> {
        let (total, phase, _delay, message) = AttemptsTracker::get_info(&self.paths)?;
        Ok((total, phase, message))
    }

    /// Check if system is currently locked
    pub fn is_locked(&self) -> Result<bool> {
        let (total, phase, _delay, _message) = AttemptsTracker::get_info(&self.paths)?;
        
        // Locked if Phase 3 with MAX_TOTAL_ATTEMPTS
        if phase == AttemptPhase::Lock && total >= MAX_TOTAL_ATTEMPTS {
            return Ok(true);
        }

        // Also check if lockout is active
        Ok(LockoutManager::remaining_lockout_time(&self.paths)?.is_some())
    }

    /// Get remaining time until unlock (if locked)
    pub fn remaining_lockout_time(&self) -> Result<Option<u64>> {
        LockoutManager::remaining_lockout_time(&self.paths)
    }
}