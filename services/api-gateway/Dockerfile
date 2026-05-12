# Build stage
FROM rust:1.83 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY services/api-gateway/Cargo.toml ./services/api-gateway/
COPY common-libs/Cargo.toml ./common-libs/
COPY common-libs/build.rs ./common-libs/

# Create dummy services to cache dependencies
RUN mkdir -p services/auth-service/src services/product-service/src services/order-service/src services/api-gateway/src common-libs/src && \
    echo "fn main() {}" > services/auth-service/src/main.rs && \
    echo "fn main() {}" > services/product-service/src/main.rs && \
    echo "fn main() {}" > services/order-service/src/main.rs && \
    echo "fn main() {}" > services/api-gateway/src/main.rs && \
    echo "" > common-libs/src/lib.rs

# Add dummy Cargo.toml for other services
RUN echo '[package]\nname = "auth-service"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]' > services/auth-service/Cargo.toml && \
    echo '[package]\nname = "product-service"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]' > services/product-service/Cargo.toml && \
    echo '[package]\nname = "order-service"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]' > services/order-service/Cargo.toml

# Copy proto files for common-libs
COPY common-libs/proto ./common-libs/proto

# Build dependencies only
RUN cargo build --release -p api-gateway

# Remove dummy source and artifacts
RUN rm -rf services/*/src common-libs/src target/release/deps/api_gateway* target/release/deps/libapi_gateway* target/release/api-gateway*

# Copy actual source code
COPY services/api-gateway/src ./services/api-gateway/src
COPY common-libs/src ./common-libs/src

# Build the actual application
RUN cargo build --release -p api-gateway

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway

# Create a non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["api-gateway"]
