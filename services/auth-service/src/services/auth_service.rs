use common_libs::proto::auth::auth_server::Auth;
use common_libs::proto::auth::{RegisterRequest, RegisterResponse};
use tonic::{Request, Response, Status};

use crate::config::AppState;

pub struct MyAuth {
    app_state: AppState,
}

impl MyAuth {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl Auth for MyAuth {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let inner = request.into_inner();
        let email = inner.email;
        let password = inner.password;

        if password.is_empty() || email.is_empty() {
            return Err(Status::invalid_argument("Email and password are required"));
        }

        // Validate email format (simple regex check)
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !email_regex.is_match(&email) {
            return Err(Status::invalid_argument("Invalid email format"));
        }

        // Check if the email already exists in the database (not implemented here)
        // If it does, return an error
        let user = sqlx::query!("SELECT * FROM users WHERE email = ?", email)
            .fetch_optional(&self.app_state.db)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if user.is_some() {
            return Err(Status::already_exists("Email already registered"));
        }

        // Check password is match from database (not implemented here)

        // Hash the password and store the user in the database (not implemented here)
        // For demonstration, we assume registration is always successful

        Ok(Response::new(RegisterResponse {
            success: true,
            message: "Registered successfully".into(),
        }))
    }
}
