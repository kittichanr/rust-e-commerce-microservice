#!/bin/bash

# Build Podman images for all services
# This script builds images that will be used by Kubernetes

set -e

echo "Building Podman images for e-commerce microservices..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the root directory (two levels up from this script)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

cd "$ROOT_DIR"

# Build auth-service
echo -e "${BLUE}Building auth-service...${NC}"
podman build -t ecommerce-auth-service:latest -f services/auth-service/Dockerfile .
echo -e "${GREEN}✓ auth-service built${NC}"

# Build product-service
echo -e "${BLUE}Building product-service...${NC}"
podman build -t ecommerce-product-service:latest -f services/product-service/Dockerfile .
echo -e "${GREEN}✓ product-service built${NC}"

# Build order-service
echo -e "${BLUE}Building order-service...${NC}"
podman build -t ecommerce-order-service:latest -f services/order-service/Dockerfile .
echo -e "${GREEN}✓ order-service built${NC}"

echo -e "${GREEN}All images built successfully!${NC}"
echo ""
echo "You can now deploy to Kubernetes with:"
echo "  kubectl apply -k infrastructure/k8s/"
echo "Or use Skaffold for development:"
echo "  cd infrastructure/k8s && skaffold dev"
