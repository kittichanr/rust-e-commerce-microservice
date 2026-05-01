use std::env;

use auth_service::{services::auth_service::MyAuth, state::AppState};
use common_libs::proto::auth::auth_server::AuthServer;
use serde::Deserialize;
use tonic::transport::Server;

#[derive(Debug, Deserialize)]
struct Config {
    db_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load from .env
    dotenvy::dotenv().ok();
    let config: Config = Config {
        db_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
    };

    // Initialize the database connection pool
    let db_pool = auth_service::db::initialize_db(&config.db_url).await?;
    let app_state = AppState { db: db_pool };

    let addr = "[::1]:50051".parse()?;
    let my_auth = MyAuth::new(app_state);

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(AuthServer::new(my_auth))
        .serve(addr)
        .await?;

    Ok(())
}
