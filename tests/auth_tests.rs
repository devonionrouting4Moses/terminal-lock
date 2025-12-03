use terminal_lock_auth::{PasswordHasher, PasswordValidator};
use terminal_lock_core::Paths;

#[test]
fn test_password_validation() {
    // Valid password
    assert!(PasswordValidator::validate("Test123").is_ok());

    // Invalid - no uppercase
    assert!(PasswordValidator::validate("test123").is_err());

    // Invalid - no lowercase
    assert!(PasswordValidator::validate("TEST123").is_err());

    // Invalid - no digit
    assert!(PasswordValidator::validate("TestAbc").is_err());

    // Invalid - too short
    assert!(PasswordValidator::validate("Te1").is_err());

    // Invalid - too long
    assert!(PasswordValidator::validate("Test1234").is_err());
}

#[test]
fn test_password_hashing() {
    let salt = PasswordHasher::generate_salt();
    let password = "Test123";

    let hash1 = PasswordHasher::hash_password(password, &salt).unwrap();
    let hash2 = PasswordHasher::hash_password(password, &salt).unwrap();

    // Same password and salt should produce same hash
    assert_eq!(hash1, hash2);

    // Different salt should produce different hash
    let salt2 = PasswordHasher::generate_salt();
    let hash3 = PasswordHasher::hash_password(password, &salt2).unwrap();
    assert_ne!(hash1, hash3);
}

#[test]
fn test_password_verification() {
    let salt = PasswordHasher::generate_salt();
    let password = "Test123";
    let hash = PasswordHasher::hash_password(password, &salt).unwrap();

    // Correct password
    assert!(PasswordHasher::verify_password(password, &hash).unwrap());

    // Wrong password
    assert!(!PasswordHasher::verify_password("Wrong123", &hash).unwrap());
}

#[test]
fn test_salt_generation() {
    let salt1 = PasswordHasher::generate_salt();
    let salt2 = PasswordHasher::generate_salt();

    // Salts should be different
    assert_ne!(salt1, salt2);

    // Salts should have reasonable length
    assert!(salt1.len() >= 16);
    assert!(salt2.len() >= 16);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_temp_paths() -> (TempDir, Paths) {
        let temp_dir = TempDir::new().unwrap();
        let lock_dir = temp_dir.path().join(".terminal_lock");
        std::fs::create_dir_all(&lock_dir).unwrap();

        // Create a custom Paths instance for testing
        let paths = Paths::new().unwrap();
        (temp_dir, paths)
    }

    #[test]
    fn test_save_and_verify_password() {
        let (_temp, paths) = setup_temp_paths();
        paths.ensure_lock_dir().unwrap();

        let password = "Test123";
        PasswordHasher::save_password(&paths, password).unwrap();

        // Verify correct password
        assert!(PasswordHasher::verify_stored_password(&paths, password).unwrap());

        // Verify incorrect password
        assert!(!PasswordHasher::verify_stored_password(&paths, "Wrong123").unwrap());
    }
}