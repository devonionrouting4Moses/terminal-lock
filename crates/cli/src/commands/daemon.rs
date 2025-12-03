use terminal_lock_core::{error::Result, LockMonitor, Paths};

/// Start the lock monitor daemon
/// This runs in the background and monitors for system lock events
pub fn run(paths: &Paths) -> Result<()> {
    eprintln!("ℹ️  Starting terminal-lock daemon...");
    eprintln!();
    eprintln!("The daemon should be managed by systemd.");
    eprintln!();
    eprintln!("To start the daemon:");
    eprintln!("  systemctl --user start terminal-lock-daemon");
    eprintln!();
    eprintln!("To check status:");
    eprintln!("  systemctl --user status terminal-lock-daemon");
    eprintln!();
    eprintln!("To view logs:");
    eprintln!("  journalctl --user -u terminal-lock-daemon -f");
    eprintln!();
    eprintln!("To enable auto-start:");
    eprintln!("  systemctl --user enable terminal-lock-daemon");
    eprintln!();
    
    // Start the lock monitor in a background thread
    LockMonitor::start(paths)?;
    
    // Keep the daemon running indefinitely
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
