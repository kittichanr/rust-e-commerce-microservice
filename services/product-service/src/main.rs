use std::sync::Arc;

use common_libs::proto::product::product_server::ProductServer;
use dotenvy::from_path;
use product_service::{
    events::{consumer::ProductEventConsumer, handler::EventHandler},
    repository::product::MySqlProductRepository,
    services::product_service::MyProductService,
};
use tonic::transport::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "product_service=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    from_path("services/product-service/.env")
        .or_else(|_| from_path("services/product-service/.env"))
        .or_else(|_| from_path(".env"))
        .ok();

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    tracing::info!("Starting product service");

    // Create database pool
    tracing::info!("Connecting to database...");
    let pool = product_service::db::initialize_db(&database_url).await?;

    tracing::info!("✓ Database connection pool established");

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("✓ Database migrations completed successfully");

    // Initialize repository
    let product_repo = Arc::new(MySqlProductRepository::new(pool));

    // Initialize Kafka event consumer
    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    tracing::info!(
        "Initializing Kafka consumer with brokers: {}",
        kafka_brokers
    );

    let event_handler = Arc::new(EventHandler::new(product_repo.clone()));
    let consumer = ProductEventConsumer::new(
        &kafka_brokers,
        "product-service-consumer-group",
        event_handler,
    )?;

    // Start Kafka consumer in background
    let _consumer_handle = consumer.start();
    tracing::info!("✓ Kafka consumer started in background");

    // Initialize gRPC service
    let product_service = MyProductService::new(product_repo);

    // Bind to all interfaces (0.0.0.0) to accept connections from other containers
    let addr = "0.0.0.0:50052".parse()?;
    tracing::info!("Product service listening on {}", addr);
    println!("Product gRPC server listening on {}", addr);

    // Start gRPC server
    Server::builder()
        .add_service(ProductServer::new(product_service))
        .serve(addr)
        .await?;

    Ok(())
}
