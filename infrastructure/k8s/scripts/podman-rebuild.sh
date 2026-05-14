#!/bin/bash

# Rebuild a specific service with Podman and restart in Kubernetes
# Usage: ./podman-rebuild.sh <service-name>
# Example: ./podman-rebuild.sh auth-service

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SERVICE=$1

if [ -z "$SERVICE" ]; then
    echo -e "${RED}Error: Service name required${NC}"
    echo "Usage: $0 <service-name>"
    echo ""
    echo "Available services:"
    echo "  - auth-service"
    echo "  - product-service"
    echo "  - order-service"
    exit 1
fi

# Validate service name
case $SERVICE in
    auth-service|product-service|order-service)
        ;;
    *)
        echo -e "${RED}Error: Invalid service name: $SERVICE${NC}"
        echo "Must be one of: auth-service, product-service, order-service"
        exit 1
        ;;
esac

# Get the root directory (two levels up from this script)
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

echo -e "${BLUE}Rebuilding $SERVICE with Podman...${NC}"

# Check if minikube is running
if ! minikube status &> /dev/null; then
    echo -e "${RED}Error: Minikube is not running${NC}"
    echo "Start it with: minikube start --driver=podman"
    exit 1
fi

# Set up Minikube's Podman environment
echo -e "${BLUE}Configuring Podman environment...${NC}"
eval $(minikube podman-env)

# Build the service
cd "$ROOT_DIR"
echo -e "${BLUE}Building image for $SERVICE...${NC}"
podman build -t ecommerce-$SERVICE:latest -f services/$SERVICE/Dockerfile .
echo -e "${GREEN}✓ Image built${NC}"

# Restart the deployment in Kubernetes
echo -e "${BLUE}Restarting deployment in Kubernetes...${NC}"
kubectl rollout restart deployment/$SERVICE -n ecommerce
echo -e "${GREEN}✓ Deployment restarted${NC}"

# Wait for rollout to complete
echo -e "${BLUE}Waiting for rollout to complete...${NC}"
kubectl rollout status deployment/$SERVICE -n ecommerce
echo -e "${GREEN}✓ Rollout complete${NC}"

echo ""
echo -e "${GREEN}$SERVICE successfully rebuilt and redeployed!${NC}"
echo ""
echo "To view logs:"
echo "  kubectl logs -f deployment/$SERVICE -n ecommerce"
