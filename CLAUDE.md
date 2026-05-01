# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based e-commerce microservice architecture using gRPC for inter-service communication. The project uses a Cargo workspace structure with shared protocol buffer definitions in `common-libs`.

## Architecture

### Workspace Structure

- **services/** - Individual microservices (currently only auth-service is active)
- **common-libs/** - Shared library containing:
  - Protocol buffer definitions (`proto/`)
  - Generated gRPC code (via `tonic-prost-build`)
  - Common utilities and types
- **infrastructure/** - Infrastructure configuration (docker-compose, etc.)

### Protocol Buffers and Code Generation

- Proto files are defined in `common-libs/proto/`
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

## Build and Development Commands

### Building

```bash
# Build entire workspace
cargo build

# Build specific service
cargo build -p auth-service

# Build common-libs (regenerates proto code)
cargo build -p common-libs

# Build with release optimizations
cargo build --release
```

### Running Services

```bash
# Run auth-service
cargo run -p auth-service --bin main

# The auth-service listens on [::1]:50051 by default
```

### Testing

```bash
# Run all tests in workspace
cargo test

# Run tests for specific package
cargo test -p auth-service
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

The project is in early development with:
- Auth service implementing basic user registration with email validation
- gRPC server infrastructure in place
- Shared proto definitions architecture established

Commented-out workspace members (product-service, order-service, api-gateway) indicate planned future services.
