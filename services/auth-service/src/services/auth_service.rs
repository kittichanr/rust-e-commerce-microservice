use chrono::{Duration, Utc};
use common_libs::proto::auth::auth_server::Auth;
use common_libs::proto::auth::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    RegisterResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::config::Config;
use crate::models::{CreateRefreshTokenInput, CreateUserInput};
use crate::repositories::{RefreshTokenRepository, UserRepository};
use crate::utils;

pub struct MyAuth {
    user_repo: Box<dyn UserRepository>,
    refresh_token_repo: Box<dyn RefreshTokenRepository>,
    config: Arc<Config>,
}

impl MyAuth {
    pub fn new(
        user_repo: Box<dyn UserRepository>,
        refresh_token_repo: Box<dyn RefreshTokenRepository>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            config,
        }
    }
}

#[tonic::async_trait]
impl Auth for MyAuth {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let inner = request.into_inner();
        let email = inner.email.trim().to_lowercase();
        let password = inner.password;

        // Validate inputs
        if password.is_empty() || email.is_empty() {
            return Err(Status::invalid_argument("email and password are required"));
        }

        // Validate email format using regex
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
            .map_err(|e| Status::internal(format!("regex error: {}", e)))?;

        if !email_regex.is_match(&email) {
            return Err(Status::invalid_argument("invalid email format"));
        }

        // Validate password strength (minimum 8 characters for security)
        if password.len() < 8 {
            return Err(Status::invalid_argument(
                "password must be at least 8 characters",
            ));
        }

        // Hash password using Argon2id
        let password_hash = utils::hash_password(&password).map_err(Status::from)?;

        // Create user via repository
        let create_input = CreateUserInput {
            email: email.clone(),
            password: password_hash,
        };

        let user = self
            .user_repo
            .create(create_input)
            .await
            .map_err(Status::from)?;

        tracing::info!("User registered successfully: {} (id: {})", email, user.id);

        Ok(Response::new(RegisterResponse {
            success: true,
            message: "registered successfully".into(),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let inner = request.into_inner();
        let email = inner.email.trim().to_lowercase();
        let password = inner.password;

        // Validate inputs
        if password.is_empty() || email.is_empty() {
            return Err(Status::invalid_argument("email and password are required"));
        }

        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(Status::from)?
            .ok_or_else(|| Status::unauthenticated("invalid email or password"))?;

        // Check if user is active
        if !user.is_active {
            return Err(Status::unauthenticated("account is deactivated"));
        }

        // Verify password
        let password_valid =
            utils::verify_password(&password, &user.password_hash).map_err(Status::from)?;

        if !password_valid {
            return Err(Status::unauthenticated("invalid email or password"));
        }

        // Generate access token (short-lived, e.g., 15 minutes)
        let access_token = utils::generate_access_token(
            user.id,
            &user.email,
            &self.config.jwt_secret,
            self.config.jwt_access_expiry_minutes,
        )
        .map_err(Status::from)?;

        // Generate refresh token (long-lived, e.g., 7 days)
        let token_id = Uuid::now_v7();
        let refresh_token_string = utils::generate_refresh_token(
            user.id,
            token_id,
            &self.config.jwt_secret,
            self.config.jwt_refresh_expiry_days,
        )
        .map_err(Status::from)?;

        // Hash the refresh token for storage
        let token_hash = utils::hash_token(&refresh_token_string).map_err(Status::from)?;

        // Store refresh token in database
        let expires_at = Utc::now() + Duration::days(self.config.jwt_refresh_expiry_days);
        let create_token_input = CreateRefreshTokenInput {
            user_id: user.id,
            token_hash,
            expires_at,
        };

        self.refresh_token_repo
            .create(create_token_input)
            .await
            .map_err(|e| Status::internal(format!("failed to store refresh token: {}", e)))?;

        tracing::info!("User logged in successfully: {} (id: {})", email, user.id);

        Ok(Response::new(LoginResponse {
            success: true,
            message: "login successful".into(),
            access_token,
            refresh_token: refresh_token_string,
            expires_in: self.config.jwt_access_expiry_minutes * 60, // convert to seconds
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let inner = request.into_inner();
        let refresh_token_string = inner.refresh_token;

        // Validate input
        if refresh_token_string.is_empty() {
            return Err(Status::invalid_argument("refresh token is required"));
        }

        // Validate and decode the refresh token
        let claims = utils::validate_refresh_token(&refresh_token_string, &self.config.jwt_secret)
            .map_err(|_| Status::unauthenticated("invalid or expired refresh token"))?;

        // Parse user_id from claims
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| Status::internal("invalid user id in token"))?;

        // Hash the refresh token to look it up in the database
        let token_hash = utils::hash_token(&refresh_token_string).map_err(Status::from)?;

        // Find the refresh token in the database
        let stored_token = self
            .refresh_token_repo
            .find_by_token_hash(&token_hash)
            .await
            .map_err(|e| Status::internal(format!("database error: {}", e)))?
            .ok_or_else(|| Status::unauthenticated("refresh token not found"))?;

        // Verify the token is valid (not revoked and not expired)
        if !stored_token.is_valid() {
            return Err(Status::unauthenticated(
                "refresh token is invalid or revoked",
            ));
        }

        // Verify the token belongs to the correct user
        if stored_token.user_id != user_id {
            return Err(Status::unauthenticated("refresh token does not match user"));
        }

        // Find the user
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(Status::from)?
            .ok_or_else(|| Status::unauthenticated("user not found"))?;

        // Check if user is still active
        if !user.is_active {
            return Err(Status::unauthenticated("account is deactivated"));
        }

        // Revoke the old refresh token for security (token rotation)
        self.refresh_token_repo
            .revoke(stored_token.id)
            .await
            .map_err(|e| Status::internal(format!("failed to revoke old token: {}", e)))?;

        // Generate new access token
        let access_token = utils::generate_access_token(
            user.id,
            &user.email,
            &self.config.jwt_secret,
            self.config.jwt_access_expiry_minutes,
        )
        .map_err(Status::from)?;

        // Generate new refresh token
        let new_token_id = Uuid::now_v7();
        let new_refresh_token_string = utils::generate_refresh_token(
            user.id,
            new_token_id,
            &self.config.jwt_secret,
            self.config.jwt_refresh_expiry_days,
        )
        .map_err(Status::from)?;

        // Hash and store the new refresh token
        let new_token_hash = utils::hash_token(&new_refresh_token_string).map_err(Status::from)?;
        let expires_at = Utc::now() + Duration::days(self.config.jwt_refresh_expiry_days);
        let create_token_input = CreateRefreshTokenInput {
            user_id: user.id,
            token_hash: new_token_hash,
            expires_at,
        };

        self.refresh_token_repo
            .create(create_token_input)
            .await
            .map_err(|e| Status::internal(format!("failed to store new refresh token: {}", e)))?;

        tracing::info!(
            "Token refreshed successfully for user: {} (id: {})",
            user.email,
            user.id
        );

        Ok(Response::new(RefreshTokenResponse {
            success: true,
            message: "token refreshed successfully".into(),
            access_token,
            refresh_token: new_refresh_token_string,
            expires_in: self.config.jwt_access_expiry_minutes * 60, // convert to seconds
        }))
    }
}
