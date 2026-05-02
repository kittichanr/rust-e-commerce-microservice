use async_trait::async_trait;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::domain::errors::AppError;
use crate::models::{CreateUserInput, User};

/// User repository trait for database operations
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn create(&self, input: CreateUserInput) -> Result<User, AppError>;
    async fn update_password(&self, id: Uuid, password_hash: String) -> Result<(), AppError>;
    async fn deactivate(&self, id: Uuid) -> Result<(), AppError>;
}

/// MySQL implementation of UserRepository
pub struct MySqlUserRepository {
    pool: MySqlPool,
}

impl MySqlUserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for MySqlUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: Uuid", email, password_hash, is_active as "is_active: bool", created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: Uuid", email, password_hash, is_active as "is_active: bool", created_at, updated_at
            FROM users
            WHERE email = ?
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn create(&self, input: CreateUserInput) -> Result<User, AppError> {
        // Validate email is not empty (should be validated earlier, but defense in depth)
        if input.email.trim().is_empty() {
            return Err(AppError::Validation("email cannot be empty".to_string()));
        }

        // Validate password_hash is not empty
        if input.password.trim().is_empty() {
            return Err(AppError::Validation(
                "password hash cannot be empty".to_string(),
            ));
        }

        // Check if email already exists
        let existing_user = self.find_by_email(&input.email).await?;
        if existing_user.is_some() {
            return Err(AppError::Conflict(format!(
                "user with email '{}' already exists",
                input.email
            )));
        }

        // Generate time-ordered UUID for better indexing performance
        let user_id = Uuid::now_v7();

        // Insert user - SQLx automatically converts Uuid to BINARY(16)
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash)
            VALUES (?, ?, ?)
            "#,
            user_id,
            input.email,
            input.password,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // Handle duplicate email constraint violation
            if let sqlx::Error::Database(db_err) = &e
                && db_err.code().as_deref() == Some("23000")
            {
                // MySQL duplicate entry error
                return AppError::Conflict(format!(
                    "user with email '{}' already exists",
                    input.email
                ));
            }
            AppError::from(e)
        })?;

        // Fetch and return the created user
        self.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Internal("failed to fetch created user".to_string()))
    }

    async fn update_password(&self, id: Uuid, password_hash: String) -> Result<(), AppError> {
        if password_hash.trim().is_empty() {
            return Err(AppError::Validation(
                "password hash cannot be empty".to_string(),
            ));
        }

        let result = sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            password_hash,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("user with id {} not found", id)));
        }

        Ok(())
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET is_active = FALSE, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("user with id {} not found", id)));
        }

        Ok(())
    }
}
