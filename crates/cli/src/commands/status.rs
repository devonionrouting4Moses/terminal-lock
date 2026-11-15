use colored::*;
use chrono::Utc;
use serde_json;
use terminal_lock_auth::{lockout::{AttemptsTracker, LockoutManager}, password::PasswordHasher};
use terminal_lock_config;
use terminal_lock_core::{constants::*, error::Result, Paths, types::ColorScheme};
use terminal_lock_ui::{Table, create_status_box, create_section_header, ColorPalette};
use crate::ipc_helper;
use std::path::PathBuf;

pub fn run(paths: &Paths) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════════════════╗".blue().bold());
    println!("{}", "║        Terminal Lock - Status Report                  ║".blue().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════╝".blue().bold());
    println!();

    // Load preferences from database via IPC (delegated to config crate)
    let (name, color_scheme_str, animations, _, quote_source_str, greeting_style_str) = 
        match ipc_helper::get_preferences(1) {
            Ok(Some(prefs)) => prefs,
            _ => ("User".to_string(), "vibrant".to_string(), true, "matrix".to_string(), "all".to_string(), "dynamic".to_string()),
        };
    
    let color_scheme = terminal_lock_config::parse_color_scheme(&color_scheme_str);
    let palette = ColorPalette::new(color_scheme);

    // Configuration status - using beautiful table
    let db_path = paths.lock_dir().join("sessions.db");
    let user_id = unsafe { libc::getuid() } as u32;
    let has_password = PasswordHasher::has_credentials_in_db(&db_path, user_id).unwrap_or(false);
    
    if !has_password {
        println!("  {} Password NOT configured (run setup)", "✗".red());
        return Ok(());
    }

    let anim_status = if animations { "true" } else { "false" };
    println!("{}", create_section_header("Configuration"));
    println!();
    
    let table = Table::new(ColorPalette::new(color_scheme))
        .with_title("System Configuration")
        .with_headers(vec!["Setting", "Value"])
        .add_row(vec!["Password", "✓ Configured"])
        .add_row(vec!["Display Name", &name])
        .add_row(vec!["Color Scheme", &color_scheme_str])
        .add_row(vec!["Animations", anim_status])
        .add_row(vec!["Quote Source", &quote_source_str])
        .add_row(vec!["Greeting Style", &greeting_style_str])
        .render();
    
    println!("{}", table);
    println!();

    // Daemon status - in table
    let daemon_status = if ipc_helper::is_daemon_running() {
        "✓ RUNNING"
    } else {
        "○ NOT RUNNING"
    };
    let ipc_status = if ipc_helper::is_daemon_running() {
        "✓ Enabled"
    } else {
        "ℹ Disabled (file-based)"
    };
    
    let table = Table::new(ColorPalette::new(color_scheme))
        .with_title("Daemon Status")
        .with_headers(vec!["Component", "Status"])
        .add_row(vec!["Daemon", daemon_status])
        .add_row(vec!["IPC", ipc_status])
        .render();
    println!("{}", table);
    println!();

    // Session status - Query daemon for all active sessions
    let shell_pid = std::env::var("TERMINAL_LOCK_SESSION_PID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| {
            std::env::var("PPID")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or_else(|| std::process::id())
        });
    
    match ipc_helper::get_session_status(None) {
        Ok(sessions) => {
            if !sessions.is_empty() {
                let now = std::time::SystemTime::now();
                let mut session_rows = Vec::new();
                
                // Add current shell session
                let current_shell_session = sessions.iter().find(|s| s.shell_pid == shell_pid);
                if let Some(session) = current_shell_session {
                    let time_remaining = session.expires_at
                        .duration_since(now)
                        .map(|d| d.as_secs() / 60)
                        .unwrap_or(0);
                    session_rows.push(vec![
                        "This shell".to_string(),
                        format!("{} minutes", time_remaining),
                    ]);
                } else {
                    session_rows.push(vec![
                        "This shell".to_string(),
                        "No active session".to_string(),
                    ]);
                }
                
                // Add other sessions
                for session in sessions.iter().filter(|s| s.shell_pid != shell_pid) {
                    let time_remaining = session.expires_at
                        .duration_since(now)
                        .map(|d| d.as_secs() / 60)
                        .unwrap_or(0);
                    session_rows.push(vec![
                        format!("PID {}", session.shell_pid),
                        format!("{} minutes", time_remaining),
                    ]);
                }
                
                let mut table = Table::new(ColorPalette::new(color_scheme))
                    .with_title("Active Sessions")
                    .with_headers(vec!["Terminal", "Time Remaining"]);
                
                for row in session_rows {
                    table = table.add_row(vec![&row[0], &row[1]]);
                }
                
                println!("{}", table.render());
                println!();
            }
        }
        Err(_) => {}
    }

    // Per-terminal attempt and lockout status - in table
    match ipc_helper::get_session_status(None) {
        Ok(sessions) => {
            if !sessions.is_empty() {
                let mut attempt_rows = Vec::new();
                
                for session in sessions {
                    let failed_attempts = ipc_helper::get_attempt_history(session.shell_pid)
                        .unwrap_or(0);
                    
                    let lockout_file = paths.lock_dir().join(format!("lockout.{}.json", session.shell_pid));
                    
                    let status_str = if lockout_file.exists() {
                        match std::fs::read_to_string(&lockout_file) {
                            Ok(content) => {
                                match serde_json::from_str::<serde_json::Value>(&content) {
                                    Ok(obj) => {
                                        if let Some(until_secs) = obj.get("until").and_then(|v| v.as_i64()) {
                                            let now_secs = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .map(|d| d.as_secs() as i64)
                                                .unwrap_or(0);
                                            let remaining = (until_secs - now_secs).max(0);
                                            format!("🔒 LOCKED ({} failed, {} sec left)", failed_attempts, remaining)
                                        } else {
                                            format!("🔒 LOCKED ({} failed)", failed_attempts)
                                        }
                                    }
                                    Err(_) => format!("🔒 LOCKED ({} failed)", failed_attempts)
                                }
                            }
                            Err(_) => format!("🔒 LOCKED ({} failed)", failed_attempts)
                        }
                    } else if failed_attempts > 0 {
                        format!("⚠ {} failed attempt(s)", failed_attempts)
                    } else {
                        "✓ Clean".to_string()
                    };
                    
                    let pid_label = if session.shell_pid == shell_pid {
                        "This shell".to_string()
                    } else {
                        format!("PID {}", session.shell_pid)
                    };
                    
                    attempt_rows.push(vec![pid_label, status_str]);
                }
                
                let mut table = Table::new(ColorPalette::new(color_scheme))
                    .with_title("Per-Terminal Status")
                    .with_headers(vec!["Terminal", "Status"]);
                
                for row in attempt_rows {
                    table = table.add_row(vec![&row[0], &row[1]]);
                }
                
                println!("{}", table.render());
                println!();
            }
        }
        Err(_) => {}
    }

    // Security status - in table
    let lock_status = if let Some(until) = LockoutManager::is_locked(paths)? {
        let now = Utc::now();
        let remaining = (until - now).to_std().unwrap_or_default().as_secs() as i64;
        format!("🔒 LOCKED ({} sec remaining)", remaining)
    } else {
        "✓ Unlocked".to_string()
    };
    
    let table = Table::new(ColorPalette::new(color_scheme))
        .with_title("Security Status")
        .with_headers(vec!["Component", "Status"])
        .add_row(vec!["Terminal Lock", &lock_status])
        .render();
    println!("{}", table);
    println!();

    // Limits - load from database - in table
    match ipc_helper::get_constants() {
        Ok((session_ttl_secs, inactivity_timeout_secs, max_attempts_per_session, max_total_attempts, lockout_duration_secs)) => {
            let table = Table::new(ColorPalette::new(color_scheme))
                .with_title("System Limits")
                .with_headers(vec!["Parameter", "Value"])
                .add_row(vec!["Per-terminal attempts", &format!("{} (lockout: {}s)", max_attempts_per_session, lockout_duration_secs)])
                .add_row(vec!["Global max attempts", &format!("{} (laptop locks)", max_total_attempts)])
                .add_row(vec!["Session timeout", &format!("{} minutes", session_ttl_secs / 60)])
                .add_row(vec!["Inactivity timeout", &format!("{} minutes", inactivity_timeout_secs / 60)])
                .render();
            println!("{}", table);
        }
        Err(_) => {
            let table = Table::new(ColorPalette::new(color_scheme))
                .with_title("System Limits")
                .with_headers(vec!["Parameter", "Value"])
                .add_row(vec!["Per-terminal attempts", &format!("{} (lockout: {}s)", MAX_ATTEMPTS_PER_SESSION, LOCKOUT_DURATION.as_secs())])
                .add_row(vec!["Global max attempts", &format!("{} (laptop locks)", MAX_TOTAL_ATTEMPTS)])
                .add_row(vec!["Session timeout", &format!("{} minutes", SESSION_TTL.as_secs() / 60)])
                .add_row(vec!["Inactivity timeout", &format!("{} minutes", INACTIVITY_TIMEOUT.as_secs() / 60)])
                .render();
            println!("{}", table);
        }
    }
    println!();

    Ok(())
}