use std::sync::Arc;

use auth_service::{
    config::Config,
    repositories::{
        refresh_token_repository::MySqlRefreshTokenRepository, user_repository::MySqlUserRepository,
    },
    services::auth_service::MyAuth,
};
use common_libs::proto::auth::auth_server::AuthServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = Config::from_env().expect("Failed to load configuration");
    let config = Arc::new(config);

    // Initialize the database connection pool
    let db_pool = auth_service::db::initialize_db(&config.db_url).await?;

    // Initialize repositories
    let user_repo = Box::new(MySqlUserRepository::new(db_pool.clone()));
    let refresh_token_repo = Box::new(MySqlRefreshTokenRepository::new(db_pool.clone()));

    // Initialize auth service with all dependencies
    let my_auth = MyAuth::new(user_repo, refresh_token_repo, config);

    let addr = "[::1]:50051".parse()?;
    tracing::info!("Auth service listening on {}", addr);
    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(AuthServer::new(my_auth))
        .serve(addr)
        .await?;

    Ok(())
}
