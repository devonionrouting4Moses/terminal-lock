use std::thread;
use std::time::Duration;
use terminal_lock_core::{session::Session, Paths};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_temp_paths() -> (TempDir, Paths) {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join(".terminal_lock");
        std::fs::create_dir_all(&lock_dir).unwrap();
        let paths = Paths::new().unwrap();
        (temp_dir, paths)
    }

    #[test]
    fn test_session_creation() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        // Create session
        Session::create(&paths).unwrap();

        // Session should be valid immediately
        assert!(Session::is_valid(&paths).unwrap());
    }

    #[test]
    fn test_session_removal() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        // Create and remove session
        Session::create(&paths).unwrap();
        Session::remove(&paths).unwrap();

        // Session should no longer be valid
        assert!(!Session::is_valid(&paths).unwrap());
    }

    #[test]
    fn test_session_validity_check() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        // No session initially
        assert!(!Session::is_valid(&paths).unwrap());

        // Create session
        Session::create(&paths).unwrap();
        assert!(Session::is_valid(&paths).unwrap());

        // Check multiple times (should extend session)
        for _ in 0..3 {
            thread::sleep(Duration::from_millis(100));
            assert!(Session::is_valid(&paths).unwrap());
        }
    }

    #[test]
    fn test_multiple_session_checks_extend_timeout() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        Session::create(&paths).unwrap();

        // First check
        let file1 = std::fs::read_to_string(paths.session_file()).unwrap();
        
        thread::sleep(Duration::from_secs(1));

        // Second check (should extend)
        assert!(Session::is_valid(&paths).unwrap());
        let file2 = std::fs::read_to_string(paths.session_file()).unwrap();

        // Expiry times should be different (extended)
        assert_ne!(file1, file2);
    }

    #[test]
    fn test_session_file_permissions() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        Session::create(&paths).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(paths.session_file()).unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }
}