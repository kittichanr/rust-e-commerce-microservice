use serde::Deserialize;
use sqlx::MySqlPool;
use std::env;

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
        dotenvy::dotenv().ok();

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
#[derive(Debug, Clone)]
pub struct AppState {
    pub db: MySqlPool,
    // Future additions:
    // pub config: Arc<Config>,
    // pub user_repo: Arc<dyn UserRepository>,
    // pub redis: deadpool_redis::Pool,
}

impl AppState {
    pub fn new(db: MySqlPool) -> Self {
        Self { db }
    }
}
