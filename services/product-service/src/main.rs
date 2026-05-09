use sqlx::mysql::MySqlPoolOptions;
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

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    tracing::info!("Starting product service database setup");

    // Create database pool
    tracing::info!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    tracing::info!("✓ Database connection pool established");

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    tracing::info!("✓ Database migrations completed successfully");

    // Verify database setup
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products")
        .fetch_one(&pool)
        .await?;

    tracing::info!("✓ Database is ready. Current product count: {}", count);
    tracing::info!("Product service database setup complete!");

    Ok(())
}
