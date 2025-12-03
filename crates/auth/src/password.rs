use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};
// use rand_core::OsRng;
use terminal_lock_core::{constants::*, error::*, paths::Paths};

/// Password validation
pub struct PasswordValidator;

impl PasswordValidator {
    /// Validate password format
    pub fn validate(password: &str) -> Result<()> {
        if password.len() != PASSWORD_MIN_LENGTH {
            return Err(Error::InvalidPasswordFormat(
                format!("Password must be exactly {} characters", PASSWORD_MIN_LENGTH)
            ));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());

        if !has_upper {
            return Err(Error::InvalidPasswordFormat(
                "Password must contain at least one uppercase letter".into()
            ));
        }

        if !has_lower {
            return Err(Error::InvalidPasswordFormat(
                "Password must contain at least one lowercase letter".into()
            ));
        }

        if !has_digit {
            return Err(Error::InvalidPasswordFormat(
                "Password must contain at least one digit".into()
            ));
        }

        Ok(())
    }
}

/// Password hashing and verification
pub struct PasswordHasher;

impl PasswordHasher {
    /// Generate a random salt
    pub fn generate_salt() -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        let random_bytes: [u8; 16] = rng.random();
        let salt = SaltString::encode_b64(&random_bytes)
            .expect("Failed to encode salt");
        salt.to_string()
    }

    /// Hash a password with salt using Argon2
    pub fn hash_password(password: &str, salt_str: &str) -> Result<String> {
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| Error::Unknown(format!("Invalid salt: {}", e)))?;

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::Unknown(format!("Hash failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against stored hash
    pub fn verify_password(password: &str, hash_str: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash_str)
            .map_err(|e| Error::Unknown(format!("Invalid hash: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Save password hash and salt to files
    pub fn save_password(paths: &Paths, password: &str) -> Result<()> {
        PasswordValidator::validate(password)?;

        let salt = Self::generate_salt();
        let hash = Self::hash_password(password, &salt)?;

        std::fs::write(paths.salt_file(), &salt)?;
        std::fs::write(paths.pass_hash_file(), &hash)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(paths.salt_file(), std::fs::Permissions::from_mode(0o600))?;
            std::fs::set_permissions(paths.pass_hash_file(), std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load and verify password
    pub fn verify_stored_password(paths: &Paths, password: &str) -> Result<bool> {
        let hash = std::fs::read_to_string(paths.pass_hash_file())?;
        Self::verify_password(password, hash.trim())
    }

    /// Get stored password hash from file
    pub fn get_stored_hash(paths: &Paths) -> Result<String> {
        let hash = std::fs::read_to_string(paths.pass_hash_file())?;
        Ok(hash.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        assert!(PasswordValidator::validate("Abc123").is_ok());
        assert!(PasswordValidator::validate("abc123").is_err()); // No uppercase
        assert!(PasswordValidator::validate("ABC123").is_err()); // No lowercase
        assert!(PasswordValidator::validate("AbcDef").is_err()); // No digit
        assert!(PasswordValidator::validate("Ab1").is_err()); // Too short
    }

    #[test]
    fn test_password_hashing() {
        let salt = PasswordHasher::generate_salt();
        let password = "Test123";
        
        let hash = PasswordHasher::hash_password(password, &salt).unwrap();
        assert!(PasswordHasher::verify_password(password, &hash).unwrap());
        assert!(!PasswordHasher::verify_password("Wrong123", &hash).unwrap());
    }
}