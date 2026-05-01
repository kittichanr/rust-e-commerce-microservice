# Auth Service

gRPC-based authentication and user management service for the e-commerce platform.

## Features

- User registration with email validation
- Password authentication with Argon2 hashing
- JWT token generation and refresh token management
- User profile and address management
- gRPC API for inter-service communication

## Tech Stack

- **Framework**: Tonic (gRPC)
- **Database**: MySQL with SQLx
- **Password Hashing**: Argon2
- **JWT**: jsonwebtoken
- **Async Runtime**: Tokio

## Prerequisites

- Rust 1.85+
- MySQL 8.0+
- sqlx-cli (for migrations)

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features mysql
```

## Local Development Setup

### 1. Environment Configuration

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your local settings:

```env
DATABASE_URL=mysql://root:password@localhost:3306/auth_db
SERVICE_PORT=50051
JWT_SECRET=your-secret-key
```

### 2. Start Database

Using Docker Compose from the infrastructure directory:

```bash
cd ../../infrastructure
docker-compose up -d auth-db
```

### 3. Run Migrations

```bash
# From auth-service directory
sqlx migrate run
```

### 4. Run the Service

```bash
# Development mode
cargo run --bin main

# With hot reload (using cargo-watch)
cargo watch -x 'run --bin main'
```

The service will start on `[::1]:50051` by default.

## Database Migrations

See [migrations/README.md](migrations/README.md) for detailed migration instructions.

### Quick Reference

```bash
# Create new migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check status
sqlx migrate info
```

## Docker Deployment

### Build Image

```bash
# From project root
docker build -f services/auth-service/Dockerfile -t auth-service:latest .
```

### Run with Docker Compose

```bash
cd infrastructure
docker-compose up -d
```

This starts both MySQL and the auth-service with proper networking and health checks.

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Project Structure

```
auth-service/
├── src/
│   ├── main.rs           # Server setup and entry point
│   ├── server.rs         # gRPC service implementation
│   └── lib.rs            # Module exports
├── migrations/           # SQLx database migrations
├── Dockerfile            # Multi-stage Docker build
├── .env.example          # Environment template
└── README.md             # This file
```

## gRPC API

The service implements the `AuthService` defined in `common-libs/proto/auth.proto`.

### Methods

- `Register(RegisterRequest) -> RegisterResponse` - User registration
- `Login(LoginRequest) -> LoginResponse` - User authentication
- `RefreshToken(RefreshTokenRequest) -> RefreshTokenResponse` - Token refresh
- `VerifyToken(VerifyTokenRequest) -> VerifyTokenResponse` - Token validation

## Configuration

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | MySQL connection string | Required |
| `SERVICE_PORT` | gRPC server port | `50051` |
| `RUST_LOG` | Logging level | `info` |
| `JWT_SECRET` | JWT signing secret | Required |
| `JWT_EXPIRATION_HOURS` | Access token expiry | `24` |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | Refresh token expiry | `30` |

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

### SQLx Offline Mode

For Docker builds without database access:

```bash
# Generate SQLx metadata
cargo sqlx prepare

# Commit .sqlx/ directory to git
git add .sqlx
```

## Troubleshooting

### Database Connection Issues

1. Ensure MySQL is running: `docker-compose ps`
2. Check connection string in `.env`
3. Verify database exists: `mysql -u root -p -e "SHOW DATABASES;"`

### Migration Errors

```bash
# Check migration status
sqlx migrate info

# Force migration table creation
mysql -u root -p auth_db -e "CREATE TABLE IF NOT EXISTS _sqlx_migrations (...)"
```

### Port Already in Use

Change `SERVICE_PORT` in `.env` or kill the process:

```bash
lsof -ti:50051 | xargs kill -9
```

## License

Part of the Rust E-Commerce Microservice project.
