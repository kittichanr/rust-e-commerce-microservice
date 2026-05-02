use std::env;

use dotenvy::from_path;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db_url: String,
    // Add more config fields as needed:
    // pub jwt_secret: String,
    // pub jwt_expiry_hours: u64,
    // pub max_login_attempts: u32,
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
            // jwt_secret: env::var("JWT_SECRET")?,
            // jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
            //     .unwrap_or_else(|_| "24".to_string())
            //     .parse()
            //     .unwrap_or(24),
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
