use std::env;

use dotenvy::from_path;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db_url: String,
    pub jwt_secret: String,
    pub jwt_access_expiry_minutes: i64,
    pub jwt_refresh_expiry_days: i64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, env::VarError> {
        from_path("services/auth-service/.env")
            .or_else(|_| from_path("services/auth-service/.env"))
            .or_else(|_| from_path(".env"))
            .ok();

        Ok(Config {
            db_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_access_expiry_minutes: env::var("JWT_ACCESS_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            jwt_refresh_expiry_days: env::var("JWT_REFRESH_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
        })
    }
}

/// Application state shared across gRPC service handlers
#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    // Future additions:
    // pub redis: deadpool_redis::Pool,
}

impl AppState {
    pub fn new(db: MySqlPool) -> Self {
        Self { db }
    }
}
