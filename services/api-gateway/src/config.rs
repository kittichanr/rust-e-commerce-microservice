use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub services: ServiceConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub auth_service_url: String,
    pub product_service_url: String,
    pub order_service_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let settings = config::Config::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("services.auth_service_url", "http://[::1]:50051")?
            .set_default("services.product_service_url", "http://[::1]:50052")?
            .set_default("services.order_service_url", "http://[::1]:50053")?
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .prefix("GATEWAY"),
            )
            .build()?;

        settings.try_deserialize()
    }
}
