mod config;
mod error;
mod handlers;
mod middleware;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, middleware::Logger, web};
use config::Config;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "api-gateway"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    let bind_address = format!("{}:{}", config.server.host, config.server.port);

    tracing::info!("Starting API Gateway on {}", bind_address);
    tracing::info!("Auth service: {}", config.services.auth_service_url);
    tracing::info!("Product service: {}", config.services.product_service_url);
    tracing::info!("Order service: {}", config.services.order_service_url);

    let state = AppState {
        config: Arc::new(config.clone()),
    };

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(tracing_actix_web::TracingLogger::default())
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // Auth routes (no auth required)
            .configure(handlers::auth::configure)
            // API routes
            .service(
                web::scope("/api")
                    // Product routes - public GET, protected POST/PUT
                    .service(
                        web::scope("/products")
                            // Public routes
                            .route("", web::get().to(handlers::product::list_products))
                            .route("/{id}", web::get().to(handlers::product::get_product))
                            // Protected routes
                            .route(
                                "",
                                web::post().to(handlers::product::create_product).wrap(
                                    middleware::JwtAuth::new(state.config.jwt.secret.clone()),
                                ),
                            )
                            .route(
                                "/{id}",
                                web::put().to(handlers::product::update_product).wrap(
                                    middleware::JwtAuth::new(state.config.jwt.secret.clone()),
                                ),
                            ),
                    )
                    // Order routes (all protected)
                    .service(
                        web::scope("")
                            .configure(handlers::order::configure)
                            .wrap(middleware::JwtAuth::new(state.config.jwt.secret.clone())),
                    ),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
