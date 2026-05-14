#!/bin/bash

# One-command setup for local Kubernetes development
# This script builds images and deploys everything

set -e

echo "Setting up e-commerce microservices for local Kubernetes development..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Step 1: Build Docker images
echo -e "${BLUE}Step 1: Building Docker images...${NC}"
bash "$SCRIPT_DIR/build-images.sh"

echo ""

# Step 2: Deploy to Kubernetes
echo -e "${BLUE}Step 2: Deploying to Kubernetes...${NC}"
bash "$SCRIPT_DIR/deploy.sh"

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Your services are now running in Kubernetes."
echo ""
echo "To access services from your local machine, run:"
echo "  kubectl port-forward svc/auth-service 50051:50051 -n ecommerce &"
echo "  kubectl port-forward svc/product-service 50052:50052 -n ecommerce &"
echo "  kubectl port-forward svc/order-service 50053:50053 -n ecommerce &"
echo "  kubectl port-forward svc/kafka 9092:9092 -n ecommerce &"
echo ""
echo "Or use the port-forward script:"
echo "  bash infrastructure/k8s/scripts/port-forward.sh"
