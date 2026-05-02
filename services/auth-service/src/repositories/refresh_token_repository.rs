use async_trait::async_trait;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::{CreateRefreshTokenInput, RefreshToken};

/// Refresh token repository trait for database operations
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, sqlx::Error>;
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, sqlx::Error>;
    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, sqlx::Error>;
    async fn create(&self, input: CreateRefreshTokenInput) -> Result<RefreshToken, sqlx::Error>;
    async fn revoke(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), sqlx::Error>;
    async fn delete_expired(&self) -> Result<u64, sqlx::Error>;
}

/// MySQL implementation of RefreshTokenRepository
pub struct MySqlRefreshTokenRepository {
    pool: MySqlPool,
}

impl MySqlRefreshTokenRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for MySqlRefreshTokenRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as!(
            RefreshToken,
            r#"
            SELECT id as "id: Uuid", user_id as "user_id: Uuid", token_hash, expires_at, revoked_at, created_at
            FROM refresh_tokens
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        sqlx::query_as!(
            RefreshToken,
            r#"
            SELECT id as "id: Uuid", user_id as "user_id: Uuid", token_hash, expires_at, revoked_at, created_at
            FROM refresh_tokens
            WHERE token_hash = ?
            "#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_active_by_user(&self, user_id: Uuid) -> Result<Vec<RefreshToken>, sqlx::Error> {
        sqlx::query_as!(
            RefreshToken,
            r#"
            SELECT id as "id: Uuid", user_id as "user_id: Uuid", token_hash, expires_at, revoked_at, created_at
            FROM refresh_tokens
            WHERE user_id = ?
              AND revoked_at IS NULL
              AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, input: CreateRefreshTokenInput) -> Result<RefreshToken, sqlx::Error> {
        let token_id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
            token_id,
            input.user_id,
            input.token_hash,
            input.expires_at
        )
        .execute(&self.pool)
        .await?;

        // Fetch the created token
        self.find_by_id(token_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn revoke(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = CURRENT_TIMESTAMP
            WHERE user_id = ?
              AND revoked_at IS NULL
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW()
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    // Note: These tests require a test database setup
    // Use sqlx::test macro for integration tests

    #[tokio::test]
    async fn test_refresh_token_is_valid() {
        let future_time = Utc::now() + Duration::hours(24);
        let token = RefreshToken {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            token_hash: "test_hash".to_string(),
            expires_at: future_time,
            revoked_at: None,
            created_at: Utc::now(),
        };

        assert!(token.is_valid());
    }

    #[tokio::test]
    async fn test_refresh_token_expired() {
        let past_time = Utc::now() - Duration::hours(1);
        let token = RefreshToken {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            token_hash: "test_hash".to_string(),
            expires_at: past_time,
            revoked_at: None,
            created_at: Utc::now(),
        };

        assert!(!token.is_valid());
    }

    #[tokio::test]
    async fn test_refresh_token_revoked() {
        let future_time = Utc::now() + Duration::hours(24);
        let token = RefreshToken {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            token_hash: "test_hash".to_string(),
            expires_at: future_time,
            revoked_at: Some(Utc::now()),
            created_at: Utc::now(),
        };

        assert!(!token.is_valid());
    }
}
