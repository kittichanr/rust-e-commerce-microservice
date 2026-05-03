use tonic::Status;

/// Application error types for auth service domain logic
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("password hashing error: {0}")]
    PasswordHash(String),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(msg) => Status::not_found(msg),
            AppError::Validation(msg) => Status::invalid_argument(msg),
            AppError::Unauthorized(msg) => Status::unauthenticated(msg),
            AppError::Conflict(msg) => Status::already_exists(msg),
            AppError::Database(e) => {
                // Log the actual DB error for debugging but don't expose internals
                tracing::error!("Database error: {:?}", e);
                Status::internal("internal database error")
            }
            AppError::PasswordHash(msg) => {
                tracing::error!("Password hashing error: {}", msg);
                Status::internal("internal error")
            }
            AppError::JwtError(msg) => {
                tracing::error!("JWT error: {}", msg);
                Status::unauthenticated("invalid or expired token")
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                Status::internal("internal error")
            }
        }
    }
}
