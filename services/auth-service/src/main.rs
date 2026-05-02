use auth_service::{
    config::{AppState, Config},
    services::auth_service::MyAuth,
};
use common_libs::proto::auth::auth_server::AuthServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize the database connection pool
    let db_pool = auth_service::db::initialize_db(&config.db_url).await?;
    let app_state = AppState::new(db_pool);

    let addr = "[::1]:50051".parse()?;
    let my_auth = MyAuth::new(app_state);

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(AuthServer::new(my_auth))
        .serve(addr)
        .await?;

    Ok(())
}
