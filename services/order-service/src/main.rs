use std::sync::Arc;

use common_libs::proto::order::order_server::OrderServer;
use common_libs::proto::product::product_client::ProductClient;
use dotenvy::from_path;
use order_service::{
    repository::order::MySqlOrderRepository, services::order_service::MyOrderService,
};
use tonic::transport::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "order_service=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    from_path("services/order-service/.env")
        .or_else(|_| from_path("services/order-service/.env"))
        .or_else(|_| from_path(".env"))
        .ok();

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    tracing::info!("Starting order service");

    // Create database pool
    tracing::info!("Connecting to database...");
    let pool = order_service::db::initialize_db(&database_url).await?;

    tracing::info!("✓ Database connection pool established");

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("✓ Database migrations completed successfully");

    // Initialize repository
    let order_repo = Arc::new(MySqlOrderRepository::new(pool));

    // Connect to product service
    let product_service_url = std::env::var("PRODUCT_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());

    tracing::info!("Connecting to product service at {}", product_service_url);
    let product_client = ProductClient::connect(product_service_url.clone())
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to connect to product service at {}: {}",
                product_service_url,
                e
            );
            anyhow::anyhow!("Failed to connect to product service: {}", e)
        })?;
    tracing::info!("✓ Connected to product service");

    // Initialize gRPC service
    let order_service = MyOrderService::new(order_repo, product_client);

    // Bind to all interfaces (0.0.0.0) to accept connections from other containers
    let addr = "0.0.0.0:50053".parse()?;
    tracing::info!("Order service listening on {}", addr);
    println!("Order gRPC server listening on {}", addr);

    // Start gRPC server
    Server::builder()
        .add_service(OrderServer::new(order_service))
        .serve(addr)
        .await?;

    Ok(())
}
