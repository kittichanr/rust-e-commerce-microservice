use sqlx::mysql::MySqlPoolOptions;

pub async fn initialize_db(db_url: &str) -> Result<sqlx::MySqlPool, sqlx::Error> {
    // Create a connection pool
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    Ok(pool)
}
