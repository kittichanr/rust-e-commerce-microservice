#!/bin/bash

# Podman-specific setup for Kubernetes development
# This script sets up Minikube with Podman and deploys all services

set -e

echo "Setting up e-commerce microservices with Podman + Kubernetes..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if podman is installed
if ! command -v podman &> /dev/null; then
    echo -e "${RED}Podman is not installed. Please install it first:${NC}"
    echo "  brew install podman"
    exit 1
fi

# Check if minikube is installed
if ! command -v minikube &> /dev/null; then
    echo -e "${RED}Minikube is not installed. Please install it first:${NC}"
    echo "  brew install minikube"
    exit 1
fi

# Check if podman machine is running
echo -e "${BLUE}Checking Podman machine status...${NC}"
if ! podman machine list | grep -q "Currently running"; then
    echo -e "${YELLOW}Podman machine not running. Starting...${NC}"
    if ! podman machine list | grep -q "podman-machine-default"; then
        echo "Initializing Podman machine..."
        podman machine init --cpus=4 --memory=8192
    fi
    podman machine start
    echo -e "${GREEN}✓ Podman machine started${NC}"
else
    echo -e "${GREEN}✓ Podman machine is running${NC}"
fi

# Check if minikube is running
echo -e "${BLUE}Checking Minikube status...${NC}"
if ! minikube status &> /dev/null; then
    echo -e "${YELLOW}Minikube not running. Starting with Podman driver...${NC}"
    minikube start --driver=podman --cpus=4 --memory=8192
    echo -e "${GREEN}✓ Minikube started${NC}"
else
    echo -e "${GREEN}✓ Minikube is running${NC}"
fi

# Set up Minikube's Podman environment
echo -e "${BLUE}Setting up Minikube Podman environment...${NC}"
eval $(minikube podman-env)
echo -e "${GREEN}✓ Environment configured${NC}"

# Build images
echo ""
echo -e "${BLUE}Step 1: Building Docker images with Podman...${NC}"
bash "$SCRIPT_DIR/build-images.sh"

# Deploy to Kubernetes
echo ""
echo -e "${BLUE}Step 2: Deploying to Kubernetes...${NC}"
bash "$SCRIPT_DIR/deploy.sh"

echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Your services are now running in Kubernetes with Podman."
echo ""
echo "To access services from your local machine, run:"
echo "  bash infrastructure/k8s/scripts/port-forward.sh"
echo ""
echo "Useful commands:"
echo "  minikube dashboard       # Open Kubernetes dashboard"
echo "  kubectl get pods -n ecommerce"
echo "  kubectl logs -f deployment/auth-service -n ecommerce"
echo ""
echo -e "${YELLOW}Note: When rebuilding images, run 'eval \$(minikube podman-env)' first${NC}"
