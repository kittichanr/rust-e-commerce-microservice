# Kubernetes Setup for E-Commerce Microservices

This directory contains Kubernetes manifests and scripts to run the entire e-commerce microservices stack locally for development and testing.

**Using Podman?** See [PODMAN_SETUP.md](./PODMAN_SETUP.md) for Podman-specific instructions.

## Prerequisites

You need one of these local Kubernetes options installed:

### Option 1: Docker Desktop (Recommended for macOS)
- Install [Docker Desktop](https://www.docker.com/products/docker-desktop)
- Enable Kubernetes in Docker Desktop settings
- Simple, integrated with Docker, good for macOS

### Option 2: Minikube
```bash
# Install minikube
brew install minikube

# Start minikube
minikube start --cpus=4 --memory=8192 --driver=docker

# Enable ingress addon (optional)
minikube addons enable ingress
```

### Option 3: kind (Kubernetes in Docker)
```bash
# Install kind
brew install kind

# Create a cluster
kind create cluster --name ecommerce

# Load Docker images into kind
kind load docker-image ecommerce-auth-service:latest --name ecommerce
kind load docker-image ecommerce-product-service:latest --name ecommerce
kind load docker-image ecommerce-order-service:latest --name ecommerce
```

### Required Tools
```bash
# Install kubectl (if not already installed)
brew install kubectl

# Verify kubectl is working
kubectl version --client

# Optional: Install skaffold for automated development workflow
brew install skaffold

# Optional: Install k9s for a better cluster management UI
brew install k9s
```

## Architecture

The Kubernetes setup includes:

- **Namespace**: `ecommerce` - All resources are isolated in this namespace
- **Databases**: 3 MySQL instances (auth-db, product-db, order-db) with persistent volumes
- **Message Broker**: Kafka + Zookeeper for event-driven architecture
- **Microservices**:
  - auth-service (port 50051)
  - product-service (port 50052)
  - order-service (port 50053)
- **ConfigMaps**: Environment configuration
- **Secrets**: Sensitive data (passwords, JWT secrets)

## Directory Structure

```
infrastructure/k8s/
├── README.md                    # This file
├── namespace.yaml               # Namespace definition
├── configmap.yaml               # Non-sensitive configuration
├── secrets.yaml                 # Sensitive configuration (DO NOT commit with real secrets)
├── kustomization.yaml           # Kustomize config for deploying all resources
├── skaffold.yaml                # Skaffold config for automated dev workflow
├── databases/                   # Database deployments
│   ├── auth-db.yaml
│   ├── product-db.yaml
│   └── order-db.yaml
├── kafka/                       # Kafka infrastructure
│   ├── zookeeper.yaml
│   └── kafka.yaml
├── services/                    # Microservice deployments
│   ├── auth-service.yaml
│   ├── product-service.yaml
│   └── order-service.yaml
└── scripts/                     # Deployment scripts
    ├── build-images.sh          # Build all Docker images
    ├── deploy.sh                # Deploy to Kubernetes
    ├── cleanup.sh               # Remove all resources
    ├── dev-setup.sh             # One-command setup
    └── port-forward.sh          # Forward ports to localhost
```

## Quick Start

### One-Command Setup

From the project root, run:

```bash
bash infrastructure/k8s/scripts/dev-setup.sh
```

This will:
1. Build all Docker images
2. Deploy everything to Kubernetes
3. Show you the status of all pods and services

### Manual Setup

If you prefer step-by-step control:

#### 1. Build Docker Images

```bash
bash infrastructure/k8s/scripts/build-images.sh
```

For **kind**, also load images:
```bash
kind load docker-image ecommerce-auth-service:latest --name ecommerce
kind load docker-image ecommerce-product-service:latest --name ecommerce
kind load docker-image ecommerce-order-service:latest --name ecommerce
```

#### 2. Deploy to Kubernetes

```bash
# Using kustomize (built into kubectl)
kubectl apply -k infrastructure/k8s/

# Or use the deploy script
bash infrastructure/k8s/scripts/deploy.sh
```

#### 3. Set Up Port Forwarding

To access services from your local machine:

```bash
bash infrastructure/k8s/scripts/port-forward.sh
```

This forwards:
- `localhost:50051` → auth-service
- `localhost:50052` → product-service
- `localhost:50053` → order-service
- `localhost:9092` → kafka
- `localhost:3306` → auth-db
- `localhost:3307` → product-db
- `localhost:3308` → order-db

## Development Workflow

### Using Skaffold (Recommended for Active Development)

Skaffold automatically rebuilds and redeploys when you change code:

```bash
cd infrastructure/k8s
skaffold dev
```

This will:
- Watch for file changes
- Rebuild affected Docker images
- Redeploy to Kubernetes
- Stream logs from all pods
- Auto-forward ports

Press `Ctrl+C` to stop and clean up.

### Manual Development Workflow

1. Make code changes
2. Rebuild images: `bash infrastructure/k8s/scripts/build-images.sh`
3. For **kind**: Reload images into the cluster
4. Restart deployments:
   ```bash
   kubectl rollout restart deployment/auth-service -n ecommerce
   kubectl rollout restart deployment/product-service -n ecommerce
   kubectl rollout restart deployment/order-service -n ecommerce
   ```

## Monitoring and Debugging

### Check Pod Status

```bash
# View all pods
kubectl get pods -n ecommerce

# Watch pods in real-time
kubectl get pods -n ecommerce -w
```

### View Logs

```bash
# Logs for a specific pod
kubectl logs -f <pod-name> -n ecommerce

# Logs for a deployment
kubectl logs -f deployment/auth-service -n ecommerce

# Logs for all containers in a pod
kubectl logs -f <pod-name> --all-containers -n ecommerce
```

### Interactive Debugging

```bash
# Execute commands in a pod
kubectl exec -it <pod-name> -n ecommerce -- /bin/bash

# Port forward to a specific pod
kubectl port-forward <pod-name> 50051:50051 -n ecommerce
```

### Using k9s (Recommended)

If you installed k9s:

```bash
k9s -n ecommerce
```

Navigate with:
- `:pods` - View pods
- `:deployments` - View deployments
- `:logs` - View logs
- `:describe` - Describe resource
- `/` - Search

### Check Service Connectivity

```bash
# Test database connectivity
kubectl exec -it deployment/auth-service -n ecommerce -- nc -zv auth-db 3306

# Test Kafka connectivity
kubectl exec -it deployment/product-service -n ecommerce -- nc -zv kafka 29092

# Test inter-service connectivity
kubectl exec -it deployment/order-service -n ecommerce -- nc -zv product-service 50052
```

## Resource Management

### View Resources

```bash
# All resources in namespace
kubectl get all -n ecommerce

# Specific resource types
kubectl get services -n ecommerce
kubectl get deployments -n ecommerce
kubectl get configmaps -n ecommerce
kubectl get secrets -n ecommerce
kubectl get pvc -n ecommerce  # Persistent volume claims
```

### Scale Services

```bash
# Scale a deployment
kubectl scale deployment/product-service --replicas=3 -n ecommerce

# Auto-scale based on CPU
kubectl autoscale deployment/product-service --min=1 --max=5 --cpu-percent=80 -n ecommerce
```

### Update Configuration

```bash
# Edit configmap
kubectl edit configmap/ecommerce-config -n ecommerce

# Edit secrets
kubectl edit secret/ecommerce-secrets -n ecommerce

# After editing, restart affected pods
kubectl rollout restart deployment/<service-name> -n ecommerce
```

## Cleanup

### Remove All Resources

```bash
# Using the cleanup script
bash infrastructure/k8s/scripts/cleanup.sh

# Or manually
kubectl delete -k infrastructure/k8s/
```

### Reset Persistent Data

Persistent volume claims retain data even after pods are deleted. To start fresh:

```bash
kubectl delete pvc --all -n ecommerce
```

## Testing

### Test the Full Stack

Once everything is deployed and port-forwarded:

1. Test Auth Service:
```bash
# Use grpcurl to test
grpcurl -plaintext localhost:50051 list
```

2. Test Product Service:
```bash
grpcurl -plaintext localhost:50052 list
```

3. Test Order Service:
```bash
grpcurl -plaintext localhost:50053 list
```

4. Test Kafka:
```bash
# List topics
docker run --rm -it --network=host confluentinc/cp-kafka:7.5.0 \
  kafka-topics --bootstrap-server localhost:9092 --list
```

### Run Integration Tests

With services running in Kubernetes:

```bash
# Set environment variables to point to forwarded ports
export AUTH_SERVICE_URL=http://localhost:50051
export PRODUCT_SERVICE_URL=http://localhost:50052
export ORDER_SERVICE_URL=http://localhost:50053

# Run tests
cargo test --workspace
```

## Production Considerations

This setup is optimized for **local development and testing**. For production:

1. **Secrets Management**: Use external secrets managers (Vault, AWS Secrets Manager)
2. **Image Registry**: Push images to a container registry instead of using local images
3. **Resource Limits**: Tune CPU and memory limits based on actual usage
4. **Persistent Volumes**: Use proper StorageClass with backups
5. **High Availability**: Run multiple replicas of each service
6. **Ingress**: Set up proper ingress controller and TLS
7. **Monitoring**: Add Prometheus, Grafana, and alerting
8. **Logging**: Set up centralized logging (ELK stack or similar)
9. **Service Mesh**: Consider Istio or Linkerd for advanced traffic management

## Troubleshooting

### Pods Not Starting

```bash
# Check pod events
kubectl describe pod <pod-name> -n ecommerce

# Check logs
kubectl logs <pod-name> -n ecommerce
```

Common issues:
- **ImagePullBackOff**: Images not built or not loaded into kind
- **CrashLoopBackOff**: Application error, check logs
- **Init:Error**: Init container failed, often database not ready

### Database Connection Issues

```bash
# Check database is ready
kubectl get pods -n ecommerce | grep db

# Test connection from service pod
kubectl exec -it deployment/auth-service -n ecommerce -- \
  nc -zv auth-db 3306
```

### Kafka Issues

```bash
# Check Zookeeper is running
kubectl logs deployment/zookeeper -n ecommerce

# Check Kafka logs
kubectl logs deployment/kafka -n ecommerce

# Verify Kafka is healthy
kubectl exec -it deployment/kafka -n ecommerce -- \
  kafka-broker-api-versions --bootstrap-server localhost:9092
```

### Service Discovery Issues

```bash
# Check service DNS
kubectl exec -it deployment/order-service -n ecommerce -- \
  nslookup product-service

# Test service connectivity
kubectl exec -it deployment/order-service -n ecommerce -- \
  nc -zv product-service 50052
```

### Port Forwarding Not Working

```bash
# Kill all existing port-forward processes
pkill -f "kubectl port-forward"

# Check if ports are already in use
lsof -i :50051
lsof -i :50052
lsof -i :50053

# Restart port forwarding
bash infrastructure/k8s/scripts/port-forward.sh
```

## Differences from Docker Compose

| Aspect | Docker Compose | Kubernetes |
|--------|----------------|------------|
| **Networking** | Single bridge network | Service-based DNS |
| **Storage** | Named volumes | PersistentVolumeClaims |
| **Scaling** | `docker-compose up --scale` | `kubectl scale` |
| **Service Discovery** | Service name | Service name + namespace |
| **Port Access** | Direct port mapping | Port forwarding required |
| **Resource Limits** | Limited support | Full control |
| **Rolling Updates** | Restart containers | Built-in rolling updates |
| **Health Checks** | Limited | Liveness/Readiness probes |

## Useful Commands Cheat Sheet

```bash
# View everything
kubectl get all -n ecommerce

# Watch pod status
kubectl get pods -n ecommerce -w

# Logs (follow)
kubectl logs -f deployment/auth-service -n ecommerce

# Execute command in pod
kubectl exec -it <pod-name> -n ecommerce -- /bin/bash

# Port forward
kubectl port-forward svc/auth-service 50051:50051 -n ecommerce

# Describe resource
kubectl describe pod <pod-name> -n ecommerce

# Restart deployment
kubectl rollout restart deployment/auth-service -n ecommerce

# Scale deployment
kubectl scale deployment/auth-service --replicas=2 -n ecommerce

# Delete pod (will be recreated)
kubectl delete pod <pod-name> -n ecommerce

# Update image
kubectl set image deployment/auth-service auth-service=ecommerce-auth-service:v2 -n ecommerce

# Rollback deployment
kubectl rollout undo deployment/auth-service -n ecommerce

# View rollout status
kubectl rollout status deployment/auth-service -n ecommerce
```

## Next Steps

- Set up Prometheus and Grafana for monitoring
- Add Ingress controller for external access
- Implement Horizontal Pod Autoscaling
- Add NetworkPolicies for security
- Set up CI/CD pipeline for automated deployments
- Configure resource quotas and limits
- Implement pod disruption budgets
- Add service mesh (Istio/Linkerd)
