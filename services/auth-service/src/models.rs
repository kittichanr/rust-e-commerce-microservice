use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User entity stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    /// UUID stored as BINARY(16) in MySQL
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sqlx(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

/// Refresh token entity for JWT rotation and logout
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    /// UUID stored as BINARY(16) in MySQL
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
}

/// Input for creating a refresh token
#[derive(Debug)]
pub struct CreateRefreshTokenInput {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with a generated UUIDv7 (time-ordered)
    pub fn new(email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            email,
            password_hash,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

impl RefreshToken {
    /// Create a new refresh token with a generated UUIDv7
    pub fn new(user_id: Uuid, token_hash: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id,
            token_hash,
            expires_at,
            revoked_at: None,
            created_at: Utc::now(),
        }
    }

    /// Check if the refresh token is still valid
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at > Utc::now()
    }
}
