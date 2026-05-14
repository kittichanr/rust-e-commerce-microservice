#!/bin/bash

# Set up port forwarding for all services
# This allows you to access services running in Kubernetes from localhost

set -e

echo "Setting up port forwarding for e-commerce services..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo -e "${YELLOW}kubectl is not installed.${NC}"
    exit 1
fi

# Function to forward a port
forward_port() {
    local service=$1
    local port=$2
    local local_port=${3:-$port}

    echo -e "${BLUE}Forwarding $service:$port -> localhost:$local_port${NC}"
    kubectl port-forward -n ecommerce "svc/$service" "$local_port:$port" &
}

# Kill any existing port-forward processes
echo "Cleaning up existing port-forward processes..."
pkill -f "kubectl port-forward" || true
sleep 1

# Forward all service ports
forward_port "auth-service" 50051
forward_port "product-service" 50052
forward_port "order-service" 50053
forward_port "kafka" 9092
forward_port "auth-db" 3306
forward_port "product-db" 3306 3307
forward_port "order-db" 3306 3308

echo ""
echo -e "${GREEN}Port forwarding is active!${NC}"
echo ""
echo "Services are accessible at:"
echo "  - Auth Service (gRPC):    localhost:50051"
echo "  - Product Service (gRPC): localhost:50052"
echo "  - Order Service (gRPC):   localhost:50053"
echo "  - Kafka:                  localhost:9092"
echo "  - Auth DB (MySQL):        localhost:3306"
echo "  - Product DB (MySQL):     localhost:3307"
echo "  - Order DB (MySQL):       localhost:3308"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all port forwarding${NC}"

# Wait for user interrupt
wait
