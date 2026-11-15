mod commands;
mod utils;
pub mod ipc_helper;

use clap::{Parser, Subcommand};
use terminal_lock_auth::Authenticator;
use terminal_lock_core::{error::Result, types::AnimationType, LockMonitor, Paths};
use terminal_lock_ui::AnimationManager;

#[derive(Parser)]
#[command(name = "terminal-lock")]
#[command(about = "Enhanced terminal authentication system with animations", long_about = None)]
#[command(version)]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initial setup - configure password and preferences
    Setup,
    
    /// Manage preferences
    Preferences,
    
    /// Reset attempts counter and lockouts
    Reset,
    
    /// Check authentication status
    Status,
    
    /// Check if session is valid (blocks terminal if invalid)
    Check,
    
    /// Lock terminal manually
    Lock,
    
    /// Extend session (update activity timestamp)
    Extend,
    
    /// Start lock monitor daemon (monitors system lock events)
    Daemon,
    
    /// Run authentication (default when called from shell)
    Auth,
    
    /// Print help message
    Help,
}

/// Start animation daemon in background (separate process)
fn start_animation_daemon(paths: &Paths) -> Result<()> {
    // Spawn animation in a separate thread that won't interfere with terminal
    std::thread::spawn(move || {
        // Load preferences from database via IPC
        if let Ok(Some((_, _, animations, animation_type_str, _, _))) = ipc_helper::get_preferences(1) {
            if animations {
                let mut animation = AnimationManager::new();
                match animation_type_str.as_str() {
                    "matrix" => animation.start_matrix(),
                    "starfield" => animation.start_starfield(),
                    "particles" => animation.start_particles(),
                    _ => animation.start_matrix(),
                }
                // Keep animation alive indefinitely
                std::mem::forget(animation);
                
                // Keep thread alive - animations run in background
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                }
            }
        }
    });
    
    Ok(())
}

/// Start lock monitor daemon in background
fn start_lock_monitor(paths: &Paths) -> Result<()> {
    let paths_clone = paths.clone();
    
    std::thread::spawn(move || {
        // Start the lock monitor in background
        if let Err(e) = LockMonitor::start(&paths_clone) {
            eprintln!("Failed to start lock monitor: {}", e);
        }
    });
    
    Ok(())
}

/// Ensure session daemon is running
fn ensure_daemon_running() -> Result<()> {
    use terminal_lock_ipc::DaemonClient;
    
    // Try to ping daemon
    match DaemonClient::send_message(terminal_lock_ipc::DaemonRequest::Ping) {
        Ok(terminal_lock_ipc::DaemonResponse::Pong) => {
            // Daemon is running
            return Ok(());
        }
        _ => {
            // Daemon not running, try to start it via systemd
            eprintln!("⚠️  Session daemon not running. Starting...");
            
            // Try systemd first
            let output = std::process::Command::new("systemctl")
                .args(&["--user", "start", "terminal-lock-daemon"])
                .output();
            
            match output {
                Ok(status) if status.status.success() => {
                    eprintln!("✅ Daemon started via systemd");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    return Ok(());
                }
                _ => {
                    eprintln!("⚠️  Could not start daemon via systemd");
                }
            }
            
            // Fallback: try to start daemon directly
            let home = dirs::home_dir()
                .ok_or_else(|| terminal_lock_core::Error::Unknown("Cannot find home directory".into()))?;
            let daemon_path = home.join(".local/bin/terminal-lock-daemon");
            
            if daemon_path.exists() {
                std::process::Command::new(&daemon_path)
                    .spawn()
                    .ok();
                eprintln!("✅ Daemon started directly");
                std::thread::sleep(std::time::Duration::from_millis(500));
                return Ok(());
            }
            
            eprintln!("⚠️  Daemon binary not found at {:?}", daemon_path);
            eprintln!("   Falling back to file-based sessions");
            Ok(())
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    // Suppress tracing unless explicitly enabled
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    let cli = Cli::parse();
    let paths = Paths::new()?;
    paths.ensure_lock_dir()?;

    match cli.command {
        Some(Commands::Setup) => {
            commands::setup::run(&paths).await?;
        }
        Some(Commands::Preferences) => {
            commands::preferences::run(&paths).await?;
        }
        Some(Commands::Reset) => {
            commands::reset::run(&paths)?;
        }
        Some(Commands::Status) => {
            commands::status::run(&paths)?;
        }
        Some(Commands::Check) => {
            commands::check::run(&paths)?;
        }
        Some(Commands::Lock) => {
            commands::lock::run(&paths)?;
        }
        Some(Commands::Extend) => {
            commands::extend::run(&paths)?;
        }
        Some(Commands::Daemon) => {
            commands::daemon::run(&paths)?;
        }
        Some(Commands::Help) => {
            commands::help::run()?;
        }
        Some(Commands::Auth) | None => {
            // Default behavior - authenticate THIS terminal
            // IMPORTANT: Each terminal is independent
            
            if !utils::is_interactive_shell() {
                return Ok(());
            }

            // Ensure daemon is running before authentication
            let _ = ensure_daemon_running();

            let authenticator = Authenticator::new(paths.clone());
            
            // Check if THIS specific terminal has a valid session
            if !authenticator.needs_authentication()? {
                // This terminal's session is still valid (within 15 minutes)
                // Start services AFTER confirming session is valid
                start_lock_monitor(&paths)?;
                start_animation_daemon(&paths)?;
                
                // Show greeting
                commands::greeting::show_greeting(&paths).await?;
                
                // Exit - animations run in background thread, shell continues normally
                return Ok(());
            }

            // This terminal needs authentication
            // Even if other terminals are open, THIS one must authenticate
            match commands::auth::run(&paths).await {
                Ok(_) => {
                    // Authentication successful for THIS terminal
                    // Create session for THIS terminal (lasts 45 minutes)
                    
                    // NOW start services AFTER successful authentication
                    start_lock_monitor(&paths)?;
                    start_animation_daemon(&paths)?;
                    
                    // Show greeting
                    commands::greeting::show_greeting(&paths).await?;
                    
                    // Exit - animations run in background thread, shell continues normally
                    return Ok(());
                }
                Err(terminal_lock_core::error::Error::ConfigNotFound) => {
                    // Setup not done yet - allow shell to open normally
                    // User will be prompted to run setup on next terminal open
                    return Ok(());
                }
                Err(e) => {
                    // Authentication failed - block THIS terminal
                    eprintln!("Authentication failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}