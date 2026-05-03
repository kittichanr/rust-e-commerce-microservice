use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
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

/// JWT claims structure for access tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    pub sub: String,      // subject (user_id)
    pub email: String,    // user email
    pub exp: i64,         // expiration time (Unix timestamp)
    pub iat: i64,         // issued at (Unix timestamp)
}

/// JWT claims structure for refresh tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    pub sub: String,      // subject (user_id)
    pub jti: String,      // JWT ID (token id for tracking)
    pub exp: i64,         // expiration time (Unix timestamp)
    pub iat: i64,         // issued at (Unix timestamp)
}

/// Generate an access token (short-lived, e.g., 15 minutes)
pub fn generate_access_token(
    user_id: Uuid,
    email: &str,
    secret: &str,
    expiry_minutes: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiry = now + Duration::minutes(expiry_minutes);

    let claims = AccessTokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiry.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::JwtError(format!("failed to encode access token: {}", e)))
}

/// Generate a refresh token (long-lived, e.g., 7 days)
pub fn generate_refresh_token(
    user_id: Uuid,
    token_id: Uuid,
    secret: &str,
    expiry_days: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiry = now + Duration::days(expiry_days);

    let claims = RefreshTokenClaims {
        sub: user_id.to_string(),
        jti: token_id.to_string(),
        exp: expiry.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::JwtError(format!("failed to encode refresh token: {}", e)))
}

/// Generic function to validate and decode a JWT token
fn validate_token<T>(token: &str, secret: &str, token_type: &str) -> Result<T, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    let validation = Validation::new(Algorithm::HS256);

    decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::JwtError(format!("invalid {} token: {}", token_type, e)))
}

/// Validate and decode an access token
pub fn validate_access_token(
    token: &str,
    secret: &str,
) -> Result<AccessTokenClaims, AppError> {
    validate_token(token, secret, "access")
}

/// Validate and decode a refresh token
pub fn validate_refresh_token(
    token: &str,
    secret: &str,
) -> Result<RefreshTokenClaims, AppError> {
    validate_token(token, secret, "refresh")
}

/// Hash a token for storage (using SHA-256 for deterministic hashing)
/// Unlike password hashing, we need deterministic hashes for token lookup
pub fn hash_token(token: &str) -> Result<String, AppError> {
    use sha2::{Sha256, Digest};

    if token.is_empty() {
        return Err(AppError::Validation("token cannot be empty".to_string()));
    }

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();

    // Convert to hex string
    Ok(format!("{:x}", result))
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

    #[test]
    fn test_hash_token_deterministic() {
        let token = "my_refresh_token_12345";

        // Hash the same token multiple times
        let hash1 = hash_token(token).expect("hashing should succeed");
        let hash2 = hash_token(token).expect("hashing should succeed");
        let hash3 = hash_token(token).expect("hashing should succeed");

        // All hashes should be identical (deterministic)
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);

        // Hash should be a 64-character hex string (SHA-256)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let token1 = "token_1";
        let token2 = "token_2";

        let hash1 = hash_token(token1).expect("hashing should succeed");
        let hash2 = hash_token(token2).expect("hashing should succeed");

        // Different tokens should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_empty() {
        let result = hash_token("");
        assert!(result.is_err());
    }
}
