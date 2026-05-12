# API Gateway Service

RESTful API Gateway for the e-commerce microservices. Built with Actix-web, this gateway provides a unified HTTP interface for all backend services.

## Features

- RESTful HTTP endpoints for all microservices
- JWT-based authentication middleware
- gRPC client integration for backend services
- CORS support for web clients
- Request/response logging and tracing
- Health check endpoint

## Endpoints

### Health Check
```
GET /health
```

### Authentication (Public)
```
POST /auth/register
POST /auth/login
POST /auth/refresh
```

### Products
```
GET  /api/products          # Public - List all products (supports filtering via query params)
GET  /api/products/{id}     # Public - Get product by ID
POST /api/products          # Protected - Create product
PUT  /api/products/{id}     # Protected - Update product
```

**List Products Query Parameters:**
- `?category=Electronics` - Filter by category
- `?is_active=true` - Filter by active status
- `?min_price=1000&max_price=5000` - Filter by price range (in cents)
- `?search=laptop` - Search in name/description
- `?page=1&per_page=20` - Pagination

### Orders (All Protected)
```
POST /api/orders            # Create new order
GET  /api/orders            # List user's orders
GET  /api/orders/{id}       # Get order details
PUT  /api/orders/{id}/status # Update order status
```

## Configuration

Environment variables (prefix with `GATEWAY__`):

```bash
# Server
GATEWAY__SERVER__HOST=127.0.0.1
GATEWAY__SERVER__PORT=8080

# Microservice URLs
GATEWAY__SERVICES__AUTH_SERVICE_URL=http://[::1]:50051
GATEWAY__SERVICES__PRODUCT_SERVICE_URL=http://[::1]:50052
GATEWAY__SERVICES__ORDER_SERVICE_URL=http://[::1]:50053

# JWT Secret (must match auth-service)
GATEWAY__JWT__SECRET=your-secret-key
```

## Running Locally

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your configuration
# Make sure JWT_SECRET matches auth-service

# Run the gateway
cargo run -p api-gateway

# Or with specific log level
RUST_LOG=debug cargo run -p api-gateway
```

## Running with Docker

```bash
cd infrastructure
docker-compose up api-gateway
```

## Example Requests

### Register User
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123",
    "name": "John Doe"
  }'
```

### Login
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

### List Products (Public)
```bash
# List all products
curl http://localhost:8080/api/products

# Filter by category
curl "http://localhost:8080/api/products?category=Electronics"

# Search products
curl "http://localhost:8080/api/products?search=laptop"

# Filter by price range
curl "http://localhost:8080/api/products?min_price=1000&max_price=5000"

# Filter by active status
curl "http://localhost:8080/api/products?is_active=true"

# Pagination
curl "http://localhost:8080/api/products?page=1&per_page=20"

# Combine multiple filters
curl "http://localhost:8080/api/products?category=Electronics&is_active=true&min_price=500&page=1&per_page=10"
```

**Query Parameters:**
- `category` (string) - Filter by product category
- `is_active` (boolean) - Filter by active status
- `min_price` (i64) - Minimum price in cents
- `max_price` (i64) - Maximum price in cents
- `search` (string) - Search in product name/description
- `page` (u64) - Page number for pagination (default: 1)
- `per_page` (u64) - Items per page (default: 10)

### Create Order (Protected)
```bash
curl -X POST http://localhost:8080/api/orders \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -d '{
    "items": [
      {
        "product_id": "product-uuid",
        "quantity": 2,
        "price": 1000
      }
    ]
  }'
```

## Architecture

The API Gateway acts as a reverse proxy, translating HTTP/JSON requests into gRPC calls:

```
Client (HTTP/JSON)
    ↓
API Gateway (Actix-web)
    ↓
Backend Services (gRPC)
```

### Request Flow

1. Client sends HTTP request to gateway
2. Gateway validates JWT (if protected route)
3. Gateway transforms HTTP request to gRPC
4. Gateway forwards to appropriate backend service
5. Backend service processes and responds
6. Gateway transforms gRPC response to HTTP/JSON
7. Gateway returns response to client

## Security

- JWT tokens validated on protected routes
- CORS configured (adjust for production)
- Service URLs configurable per environment
- No direct database access (through services only)

## Development

Build and test:
```bash
cargo build -p api-gateway
cargo test -p api-gateway
cargo clippy -p api-gateway
```
