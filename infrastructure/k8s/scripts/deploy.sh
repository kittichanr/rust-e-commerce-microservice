#!/bin/bash

# Deploy e-commerce microservices to Kubernetes
# This script applies all Kubernetes manifests

set -e

echo "Deploying e-commerce microservices to Kubernetes..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the k8s directory
K8S_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$K8S_DIR"

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo -e "${YELLOW}kubectl is not installed. Please install kubectl first.${NC}"
    exit 1
fi

# Apply all manifests using kustomize
echo -e "${BLUE}Applying Kubernetes manifests...${NC}"
kubectl apply -k .

echo -e "${GREEN}✓ All manifests applied${NC}"
echo ""
echo "Checking deployment status..."
echo ""

# Wait a moment for resources to be created
sleep 3

# Show pod status
echo -e "${BLUE}Pods:${NC}"
kubectl get pods -n ecommerce

echo ""
echo -e "${BLUE}Services:${NC}"
kubectl get services -n ecommerce

echo ""
echo -e "${GREEN}Deployment complete!${NC}"
echo ""
echo "To check pod logs:"
echo "  kubectl logs -f <pod-name> -n ecommerce"
echo ""
echo "To forward ports for local access:"
echo "  kubectl port-forward svc/auth-service 50051:50051 -n ecommerce"
echo "  kubectl port-forward svc/product-service 50052:50052 -n ecommerce"
echo "  kubectl port-forward svc/order-service 50053:50053 -n ecommerce"
echo "  kubectl port-forward svc/kafka 9092:9092 -n ecommerce"
