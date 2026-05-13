# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based e-commerce microservice architecture using gRPC for inter-service communication. The project uses a Cargo workspace structure with shared protocol buffer definitions in `common-libs`.

## Architecture

### Workspace Structure

- **services/** - Individual microservices:
  - **auth-service** - User authentication and authorization (port 50051)
  - **product-service** - Product catalog management
  - **order-service** - Order processing and management (port 50053)
- **common-libs/** - Shared library containing:
  - Protocol buffer definitions (`proto/`)
  - Generated gRPC code (via `tonic-prost-build`)
  - Common utilities and types
- **infrastructure/** - Infrastructure configuration:
  - Docker Compose setup with MySQL databases for each service
  - Environment configuration
  - Database initialization scripts

### Protocol Buffers and Code Generation

- Proto files are defined in `common-libs/proto/`:
  - `auth.proto` - Authentication service definitions
  - `product.proto` - Product service definitions
  - `order.proto` - Order service definitions
- `common-libs/build.rs` compiles protos using `tonic-prost-build::compile_protos()`
- Generated code is available through `common_libs::proto::<service_name>` modules
- Services import generated code from `common-libs` rather than generating their own

**Important:** When adding new proto definitions or modifying existing ones:
1. Edit the `.proto` file in `common-libs/proto/`
2. Update `common-libs/build.rs` if adding new proto files
3. Rebuild `common-libs` with `cargo build -p common-libs`
4. The generated code will be available to all services

### Service Architecture Pattern

Each service follows this structure:
- `src/main.rs` - Sets up Tonic server, configures address/port, registers service implementations
- `src/server.rs` - Contains service implementation structs and gRPC trait implementations
- `src/lib.rs` - Exports modules for testing

Services implement the generated traits from `common_libs::proto::<service>::<service>_server::<Service>` using `#[tonic::async_trait]`.

### Inter-Service Communication

Services communicate using two patterns:

**Synchronous (gRPC)**:
- **Order Service → Product Service**: Order service uses `ProductClient` from `common_libs::proto::product` to validate products and check stock availability when creating orders
- Services connect to each other via the shared Docker network or localhost in development

**Asynchronous (Kafka Events)**:
- **Order Service → Product Service**: When an order is marked as DELIVERED, the order service publishes an `OrderDeliveredEvent` to Kafka, which the product service consumes to reduce stock quantities
- Events are defined in `common-libs/src/events.rs`
- Kafka topics: `order.delivered`

### Database Architecture

Each service has its own dedicated MySQL database:
- **auth-db** (port 3306): User accounts and authentication data
- **product-db** (port 3307): Product catalog and inventory
- **order-db** (port 3308): Orders and order items

This follows the microservices pattern of database-per-service for data isolation and independence.

## Build and Development Commands

### Building

```bash
# Build entire workspace
cargo build

# Build specific service
cargo build -p auth-service
cargo build -p product-service
cargo build -p order-service

# Build common-libs (regenerates proto code)
cargo build -p common-libs

# Build with release optimizations
cargo build --release
```

### Running Services

```bash
# Run auth-service
cargo run -p auth-service --bin main
# Listens on [::1]:50051 by default

# Run product-service
cargo run -p product-service --bin main
# Requires DATABASE_URL environment variable

# Run order-service
cargo run -p order-service --bin main
# Listens on [::1]:50053 by default

# Run all services with Docker Compose
cd infrastructure
docker-compose up -d

# View logs
docker-compose logs -f [service-name]

# Stop services
docker-compose down
```

### Testing

```bash
# Run all tests in workspace
cargo test

# Run tests for specific package
cargo test -p auth-service
cargo test -p product-service
cargo test -p order-service
cargo test -p common-libs

# Run specific test
cargo test <test_name>
```

### Code Quality

```bash
# Check code without building
cargo check

# Run clippy lints
cargo clippy

# Format code
cargo fmt
```

## Current State

The project is actively developed with three functional microservices:

### Auth Service
- User registration with email validation
- JWT-based authentication (access and refresh tokens)
- gRPC server on port 50051
- MySQL database backend

### Product Service
- Product catalog management (CRUD operations)
- Category management
- Stock tracking with automatic reduction on order delivery (via Kafka events)
- MySQL database backend
- gRPC service for inter-service communication
- Kafka consumer for processing order events

### Order Service
- Order creation and management
- Order status tracking (Pending, Processing, Shipped, Delivered, Cancelled)
- Integration with Product Service via gRPC client for order validation
- Order listing and retrieval
- Kafka producer for publishing order events (OrderDeliveredEvent when status changes to DELIVERED)
- MySQL database backend
- gRPC server on port 50053

### Infrastructure
- Docker Compose setup with dedicated MySQL databases for each service
- Apache Kafka and Zookeeper for event-driven architecture
- Health checks for database and Kafka dependencies
- Shared networking for inter-service communication
- Environment-based configuration

### Common Libraries
- Shared event schemas (`common-libs/src/events.rs`)
  - `OrderDeliveredEvent` - Published when order is delivered, consumed by product service to reduce stock
- Protocol buffer definitions for gRPC services
- Event topics defined in `common_libs::events::topics`

### Event-Driven Architecture

The system uses Kafka for asynchronous event processing:

**Event Flow:**
1. Order status updated to DELIVERED → Order Service publishes `OrderDeliveredEvent`
2. Product Service consumes the event and reduces stock for all items in the order
3. Stock reduction is logged and low-stock warnings are emitted if needed

See `KAFKA_SETUP.md` for detailed information about:
- Installing dependencies (cmake, pkg-config)
- Event schemas and topics
- Testing the event flow
- Monitoring and debugging

### API Gateway
- Unified REST API gateway for all services
- JWT authentication middleware
- Request routing to appropriate microservices
