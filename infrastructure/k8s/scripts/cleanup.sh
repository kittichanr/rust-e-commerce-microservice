#!/bin/bash

# Clean up all Kubernetes resources for e-commerce microservices

set -e

echo "Cleaning up e-commerce microservices from Kubernetes..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get the k8s directory
K8S_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$K8S_DIR"

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}kubectl is not installed.${NC}"
    exit 1
fi

echo -e "${BLUE}Deleting all resources in ecommerce namespace...${NC}"
kubectl delete -k . --ignore-not-found=true

echo -e "${GREEN}✓ All resources deleted${NC}"
echo ""
echo "Checking for remaining resources..."
kubectl get all -n ecommerce 2>/dev/null || echo "Namespace is clean"

echo ""
echo -e "${GREEN}Cleanup complete!${NC}"
