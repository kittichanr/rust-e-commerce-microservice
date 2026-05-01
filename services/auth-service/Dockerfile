# Build stage using cargo-chef for caching dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.85 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# Planner stage - analyze dependencies
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - compile the application
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json
# Copy source code
COPY . .
# Enable SQLx offline mode for compile-time verification without database
ENV SQLX_OFFLINE=true
# Build the auth-service binary
RUN cargo build --release --bin main -p auth-service

# Runtime stage - minimal image
FROM debian:bookworm-slim as runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary from builder
COPY --from=builder /app/target/release/main /app/auth-service
# Copy migrations for runtime database setup
COPY --from=builder /app/services/auth-service/migrations /app/migrations
# Expose gRPC port
EXPOSE 50051
# Run the service
ENTRYPOINT ["/app/auth-service"]
