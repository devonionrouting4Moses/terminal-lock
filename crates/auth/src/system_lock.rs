use std::process::Command;
use terminal_lock_core::{error::*, paths::Paths};

/// System locker - locks the laptop after too many failed attempts
pub struct SystemLocker;

impl SystemLocker {
    /// Lock the system using available system commands
    pub fn lock_system() -> Result<()> {
        // Create lock marker file so we can detect when laptop is unlocked
        let paths = Paths::new()?;
        let marker_file = paths.lock_marker_file();
        std::fs::write(&marker_file, "locked")?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &marker_file,
                std::fs::Permissions::from_mode(0o600),
            )?;
        }
        
        // Now lock the system
        #[cfg(target_os = "linux")]
        {
            Self::lock_linux()
        }

        #[cfg(target_os = "macos")]
        {
            Self::lock_macos()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(Error::Unknown("System lock not supported on this platform".into()))
        }
    }

    #[cfg(target_os = "linux")]
    fn lock_linux() -> Result<()> {
        // Try multiple methods in order of preference
        let commands = vec![
            ("loginctl", vec!["lock-session"]),
            ("gnome-screensaver-command", vec!["-l"]),
            ("xdg-screensaver", vec!["lock"]),
            ("dm-tool", vec!["lock"]),
            ("systemctl", vec!["suspend"]),
        ];

        for (cmd, args) in commands {
            if Self::command_exists(cmd) {
                let result = Command::new(cmd).args(&args).output();
                
                if result.is_ok() {
                    return Ok(());
                }
            }
        }

        Err(Error::Unknown(
            "No suitable screen lock command found. Please lock manually.".into()
        ))
    }

    #[cfg(target_os = "macos")]
    fn lock_macos() -> Result<()> {
        Command::new("pmset")
            .args(&["displaysleepnow"])
            .output()
            .map_err(|e| Error::Unknown(format!("Failed to lock macOS: {}", e)))?;

        Ok(())
    }

    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}