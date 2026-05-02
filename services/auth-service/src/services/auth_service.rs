use common_libs::proto::auth::auth_server::Auth;
use common_libs::proto::auth::{RegisterRequest, RegisterResponse};
use tonic::{Request, Response, Status};

use crate::models::CreateUserInput;
use crate::repositories::UserRepository;
use crate::utils;

pub struct MyAuth {
    user_repo: Box<dyn UserRepository>,
}

impl MyAuth {
    pub fn new(user_repo: Box<dyn UserRepository>) -> Self {
        Self { user_repo }
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
}
