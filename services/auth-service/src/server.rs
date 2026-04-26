use common_libs::proto::auth::auth_server::Auth;
use common_libs::proto::auth::{RegisterRequest, RegisterResponse};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct MyAuth {}

#[tonic::async_trait]
impl Auth for MyAuth {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let inner = request.into_inner();
        let username = inner.username;
        let password = inner.password;
        let email = inner.email;

        if username.is_empty() || password.is_empty() || email.is_empty() {
            return Err(Status::invalid_argument(
                "Username, password, and email are required",
            ));
        }

        // Validate email format (simple regex check)
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !email_regex.is_match(&email) {
            return Err(Status::invalid_argument("Invalid email format"));
        }

        // Check if the username already exists in the database (not implemented here)
        // If it does, return an error

        // Hash the password and store the user in the database (not implemented here)
        // For demonstration, we assume registration is always successful

        Ok(Response::new(RegisterResponse {
            success: true,
            message: "Registered successfully".into(),
        }))
    }
}
