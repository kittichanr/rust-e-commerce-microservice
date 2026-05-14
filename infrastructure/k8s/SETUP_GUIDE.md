# Local Kubernetes Setup Guide

This guide walks you through setting up this e-commerce microservices project on Kubernetes for local testing.

## Choose Your Local Kubernetes Environment

### For macOS (Recommended: Docker Desktop)

Docker Desktop is the easiest option for macOS users:

1. **Install Docker Desktop**
   ```bash
   # Download from: https://www.docker.com/products/docker-desktop
   # Or install with Homebrew
   brew install --cask docker
   ```

2. **Enable Kubernetes**
   - Open Docker Desktop
   - Go to Settings → Kubernetes
   - Check "Enable Kubernetes"
   - Click "Apply & Restart"
   - Wait for Kubernetes to start (green indicator)

3. **Verify Installation**
   ```bash
   kubectl version --client
   kubectl cluster-info
   ```

### Alternative: Minikube

Good for more control over the cluster configuration:

```bash
# Install
brew install minikube

# Start with sufficient resources
minikube start --cpus=4 --memory=8192 --driver=docker

# Verify
kubectl get nodes
```

### Alternative: kind (Kubernetes in Docker)

Lightweight and fast for testing:

```bash
# Install
brew install kind

# Create cluster
kind create cluster --name ecommerce

# Verify
kubectl cluster-info --context kind-ecommerce
```

## Step-by-Step Setup

### 1. Prepare Your Environment

```bash
# Navigate to project root
cd /path/to/rust-e-commerce-microservice

# Ensure you have all dependencies
brew install kubectl
```

### 2. Build Docker Images

This builds optimized Docker images for all services:

```bash
bash infrastructure/k8s/scripts/build-images.sh
```

**For kind users only**: Load images into the cluster:
```bash
kind load docker-image ecommerce-auth-service:latest --name ecommerce
kind load docker-image ecommerce-product-service:latest --name ecommerce
kind load docker-image ecommerce-order-service:latest --name ecommerce
```

### 3. Deploy to Kubernetes

Deploy all services, databases, and Kafka:

```bash
bash infrastructure/k8s/scripts/deploy.sh
```

This will create:
- Namespace: `ecommerce`
- 3 MySQL databases (auth, product, order)
- Kafka + Zookeeper
- 3 microservices (auth, product, order)

### 4. Verify Deployment

```bash
# Check all pods are running
kubectl get pods -n ecommerce

# You should see all pods in "Running" state:
# - auth-db-xxx
# - product-db-xxx
# - order-db-xxx
# - zookeeper-xxx
# - kafka-xxx
# - auth-service-xxx
# - product-service-xxx
# - order-service-xxx
```

**Wait for all pods to be Ready** (this may take 2-3 minutes on first run).

### 5. Set Up Port Forwarding

To access services from your local machine:

```bash
bash infrastructure/k8s/scripts/port-forward.sh
```

This makes services available at:
- **Auth Service**: `localhost:50051`
- **Product Service**: `localhost:50052`
- **Order Service**: `localhost:50053`
- **Kafka**: `localhost:9092`
- **Auth DB**: `localhost:3306`
- **Product DB**: `localhost:3307`
- **Order DB**: `localhost:3308`

Leave this terminal running. Open a new terminal for the next steps.

### 6. Test the Services

Install grpcurl for testing gRPC services:

```bash
brew install grpcurl
```

Test each service:

```bash
# List services in auth-service
grpcurl -plaintext localhost:50051 list

# List services in product-service
grpcurl -plaintext localhost:50052 list

# List services in order-service
grpcurl -plaintext localhost:50053 list
```

### 7. Run Your Application Tests

Now you can run your Rust tests against the Kubernetes environment:

```bash
# Set environment variables (if needed)
export AUTH_SERVICE_URL=http://localhost:50051
export PRODUCT_SERVICE_URL=http://localhost:50052
export ORDER_SERVICE_URL=http://localhost:50053

# Run tests
cargo test --workspace
```

## Development Workflow

### Quick Iteration with Skaffold (Optional)

For the fastest development experience, install Skaffold:

```bash
brew install skaffold

# Navigate to k8s directory
cd infrastructure/k8s

# Start development mode
skaffold dev
```

Skaffold will:
- Watch your code for changes
- Automatically rebuild and redeploy
- Stream logs from all services
- Auto-forward ports

Press `Ctrl+C` to stop.

### Manual Rebuild and Deploy

When you make code changes:

```bash
# Rebuild images
bash infrastructure/k8s/scripts/build-images.sh

# For kind: Reload images
kind load docker-image ecommerce-auth-service:latest --name ecommerce
# (repeat for other services if needed)

# Restart specific service
kubectl rollout restart deployment/auth-service -n ecommerce

# Or redeploy everything
bash infrastructure/k8s/scripts/deploy.sh
```

## Monitoring and Debugging

### View Logs

```bash
# Follow logs for a specific service
kubectl logs -f deployment/auth-service -n ecommerce
kubectl logs -f deployment/product-service -n ecommerce
kubectl logs -f deployment/order-service -n ecommerce

# View Kafka logs
kubectl logs -f deployment/kafka -n ecommerce
```

### Check Service Health

```bash
# Get all resources
kubectl get all -n ecommerce

# Describe a pod (shows events and issues)
kubectl describe pod <pod-name> -n ecommerce

# Execute commands inside a pod
kubectl exec -it deployment/auth-service -n ecommerce -- /bin/bash
```

### Using k9s (Recommended)

k9s provides a terminal UI for managing Kubernetes:

```bash
# Install
brew install k9s

# Launch
k9s -n ecommerce
```

In k9s:
- Type `:pods` to see pods
- Type `:svc` to see services
- Type `:deploy` to see deployments
- Press `Enter` on a pod to see logs
- Press `d` to describe a resource
- Press `?` for help

## Common Issues and Solutions

### Pods Stuck in "Pending" or "ContainerCreating"

```bash
# Check what's wrong
kubectl describe pod <pod-name> -n ecommerce
```

**Common causes**:
- Insufficient resources: Check your Docker Desktop/Minikube resource limits
- PVC not bound: Check `kubectl get pvc -n ecommerce`

### Pods in "CrashLoopBackOff"

```bash
# View logs to see the error
kubectl logs <pod-name> -n ecommerce
```

**Common causes**:
- Database not ready: Services may start before databases are ready (they should retry)
- Missing migrations: Check database initialization
- Configuration error: Check secrets and configmaps

### "ImagePullBackOff" Error

This means Kubernetes can't find the Docker image.

**Solution**:
```bash
# Rebuild images
bash infrastructure/k8s/scripts/build-images.sh

# For kind: Load images into cluster
kind load docker-image ecommerce-auth-service:latest --name ecommerce
kind load docker-image ecommerce-product-service:latest --name ecommerce
kind load docker-image ecommerce-order-service:latest --name ecommerce
```

### Port Forwarding Stops Working

```bash
# Kill existing port-forward processes
pkill -f "kubectl port-forward"

# Restart
bash infrastructure/k8s/scripts/port-forward.sh
```

### Can't Connect to Database

```bash
# Check database pod is running
kubectl get pods -n ecommerce | grep db

# Test connectivity from a service
kubectl exec -it deployment/auth-service -n ecommerce -- nc -zv auth-db 3306
```

## Clean Up

### Remove All Resources

```bash
bash infrastructure/k8s/scripts/cleanup.sh
```

### Reset Databases (Delete Persistent Data)

```bash
kubectl delete pvc --all -n ecommerce
```

### Stop Kubernetes Cluster

**Docker Desktop**: Go to Docker Desktop settings and disable Kubernetes

**Minikube**:
```bash
minikube stop
# Or delete completely
minikube delete
```

**kind**:
```bash
kind delete cluster --name ecommerce
```

## Quick Reference

```bash
# View all resources
kubectl get all -n ecommerce

# View pods
kubectl get pods -n ecommerce

# View logs
kubectl logs -f deployment/<service-name> -n ecommerce

# Restart a service
kubectl rollout restart deployment/<service-name> -n ecommerce

# Scale a service
kubectl scale deployment/<service-name> --replicas=2 -n ecommerce

# Execute commands in a pod
kubectl exec -it <pod-name> -n ecommerce -- /bin/bash

# Delete a pod (it will be recreated)
kubectl delete pod <pod-name> -n ecommerce

# Port forward
kubectl port-forward svc/<service-name> <local-port>:<service-port> -n ecommerce
```

## Next Steps

Once you have everything running:

1. **Explore the services**: Use grpcurl to interact with gRPC endpoints
2. **Test the event flow**: Create an order, mark it as delivered, see stock reduce
3. **Monitor resources**: Check CPU/memory usage with `kubectl top pods -n ecommerce`
4. **Experiment with scaling**: Try scaling services up and down
5. **Simulate failures**: Delete a pod and watch Kubernetes recover

## Getting Help

- **Kubernetes documentation**: https://kubernetes.io/docs/
- **kubectl cheat sheet**: https://kubernetes.io/docs/reference/kubectl/cheatsheet/
- **Check pod events**: `kubectl describe pod <pod-name> -n ecommerce`
- **Check service logs**: `kubectl logs -f deployment/<service> -n ecommerce`

---

Happy testing with Kubernetes!
