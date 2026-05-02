use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use uuid::Uuid;

use crate::domain::errors::AppError;

pub fn generate_id() -> Uuid {
    Uuid::now_v7()
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    if password.is_empty() {
        return Err(AppError::Validation("password cannot be empty".to_string()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::PasswordHash(e.to_string()))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    if password.is_empty() {
        return Err(AppError::Validation("password cannot be empty".to_string()));
    }

    if password_hash.is_empty() {
        return Err(AppError::Validation(
            "password hash cannot be empty".to_string(),
        ));
    }

    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::PasswordHash(format!("invalid password hash: {}", e)))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::PasswordHash(e.to_string())),
    }
}

/// Format UUID for logging/display (hyphenated format)
pub fn format_uuid(id: &Uuid) -> String {
    id.to_string()
}

/// Parse UUID from string
pub fn parse_uuid(s: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();

        // UUIDs should be unique
        assert_ne!(id1, id2);

        // UUIDs should be version 7
        assert_eq!(id1.get_version_num(), 7);
        assert_eq!(id2.get_version_num(), 7);
    }

    #[test]
    fn test_format_and_parse_uuid() {
        let original = generate_id();
        let formatted = format_uuid(&original);
        let parsed = parse_uuid(&formatted).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_hash_password() {
        let password = "secure_password_123";
        let hash = hash_password(password).expect("hashing should succeed");

        // Hash should start with Argon2id identifier
        assert!(hash.starts_with("$argon2id$"));

        // Hash should be reasonably long
        assert!(hash.len() > 50);
    }

    #[test]
    fn test_hash_password_empty() {
        let result = hash_password("");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_success() {
        let password = "my_secure_password";
        let hash = hash_password(password).expect("hashing should succeed");

        let verified = verify_password(password, &hash).expect("verification should succeed");
        assert!(verified);
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).expect("hashing should succeed");

        let verified = verify_password(wrong_password, &hash).expect("verification should succeed");
        assert!(!verified);
    }

    #[test]
    fn test_verify_password_empty_password() {
        let hash = hash_password("password").unwrap();
        let result = verify_password("", &hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_empty_hash() {
        let result = verify_password("password", "");
        assert!(result.is_err());
    }
}
