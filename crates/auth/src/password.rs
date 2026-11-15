use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};
// use rand_core::OsRng;
use terminal_lock_core::{constants::*, error::*, paths::Paths};
use std::path::PathBuf;
use rusqlite::OptionalExtension;

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
        let random_bytes: [u8; 32] = rng.random();
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


    /// Save password to database (new database-backed method)
    pub fn save_password_to_db(
        db_path: &PathBuf,
        user_id: u32,
        password: &str,
    ) -> Result<String> {
        PasswordValidator::validate(password)?;

        let salt = Self::generate_salt();
        let hash = Self::hash_password(password, &salt)?;

        // Use dynamic import to avoid circular dependencies
        use rusqlite::Connection;
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Unknown(format!("Failed to open database: {}", e)))?;

        let credential_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::Unknown(format!("Time error: {}", e)))?
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO credentials (id, user_id, hashed_password, salt, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                &credential_id,
                user_id as i32,
                &hash,
                &salt,
                now,
            ],
        )
        .map_err(|e| Error::Unknown(format!("Failed to save credentials: {}", e)))?;

        Ok(credential_id)
    }

    /// Verify password against database credentials
    pub fn verify_password_from_db(
        db_path: &PathBuf,
        user_id: u32,
        password: &str,
    ) -> Result<bool> {
        use rusqlite::Connection;
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Unknown(format!("Failed to open database: {}", e)))?;

        let result: Option<String> = conn.query_row(
            "SELECT hashed_password FROM credentials WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![user_id as i32],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| Error::Unknown(format!("Database query failed: {}", e)))?;

        match result {
            Some(hash) => Self::verify_password(password, hash.as_str()),
            None => Err(Error::Unknown("No credentials found for user".into())),
        }
    }

    /// Get stored password hash from database
    pub fn get_stored_hash_from_db(db_path: &PathBuf, user_id: u32) -> Result<String> {
        use rusqlite::Connection;
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Unknown(format!("Failed to open database: {}", e)))?;

        let hash: String = conn.query_row(
            "SELECT hashed_password FROM credentials WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![user_id as i32],
            |row| row.get(0),
        )
        .map_err(|e| Error::Unknown(format!("Failed to get credentials: {}", e)))?;

        Ok(hash)
    }

    /// Check if user has credentials in database
    pub fn has_credentials_in_db(db_path: &PathBuf, user_id: u32) -> Result<bool> {
        use rusqlite::Connection;
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Unknown(format!("Failed to open database: {}", e)))?;

        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM credentials WHERE user_id = ?1",
            rusqlite::params![user_id as i32],
            |row| row.get(0),
        )
        .map_err(|e| Error::Unknown(format!("Database query failed: {}", e)))?;

        Ok(count > 0)
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