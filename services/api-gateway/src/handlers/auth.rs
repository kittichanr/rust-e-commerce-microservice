use crate::{error::GatewayError, AppState};
use actix_web::{web, HttpResponse};
use common_libs::proto::auth::{
    auth_client::AuthClient, LoginRequest, RefreshTokenRequest, RegisterRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub async fn register(
    state: web::Data<AppState>,
    input: web::Json<RegisterInput>,
) -> Result<HttpResponse, GatewayError> {
    let mut client = AuthClient::connect(state.config.services.auth_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Auth service: {}", e)))?;

    let request = tonic::Request::new(RegisterRequest {
        email: input.email.clone(),
        password: input.password.clone(),
    });

    let response = client.register(request).await?.into_inner();

    Ok(HttpResponse::Created().json(RegisterResponse {
        success: response.success,
        message: response.message,
    }))
}

pub async fn login(
    state: web::Data<AppState>,
    input: web::Json<LoginInput>,
) -> Result<HttpResponse, GatewayError> {
    let mut client = AuthClient::connect(state.config.services.auth_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Auth service: {}", e)))?;

    let request = tonic::Request::new(LoginRequest {
        email: input.email.clone(),
        password: input.password.clone(),
    });

    let response = client.login(request).await?.into_inner();

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: response.success,
        message: response.message,
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_in: response.expires_in,
    }))
}

pub async fn refresh_token(
    state: web::Data<AppState>,
    input: web::Json<RefreshInput>,
) -> Result<HttpResponse, GatewayError> {
    let mut client = AuthClient::connect(state.config.services.auth_service_url.clone())
        .await
        .map_err(|e| GatewayError::ServiceUnavailable(format!("Auth service: {}", e)))?;

    let request = tonic::Request::new(RefreshTokenRequest {
        refresh_token: input.refresh_token.clone(),
    });

    let response = client.refresh_token(request).await?.into_inner();

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: response.success,
        message: response.message,
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires_in: response.expires_in,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token)),
    );
}
