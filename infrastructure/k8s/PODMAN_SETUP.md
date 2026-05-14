# Kubernetes Setup with Podman

This guide is for users running Podman instead of Docker. The setup requires a few extra steps to make images available to Kubernetes.

## Prerequisites

```bash
# Install required tools
brew install podman
brew install kubectl
brew install minikube
```

## Recommended Approach: Minikube with Podman

This is the easiest way to use Podman with Kubernetes locally.

### 1. Start Minikube with Podman Driver

```bash
# Initialize podman machine (if not already done)
podman machine init --cpus=4 --memory=8192

# Start podman machine
podman machine start

# Start minikube with podman driver
minikube start --driver=podman --cpus=4 --memory=8192
```

### 2. Use Minikube's Podman Environment

This makes images built with Podman available to Minikube:

```bash
# Set environment to use minikube's podman
eval $(minikube podman-env)

# Verify you're using the right environment
podman images
```

### 3. Build Images

Now build your images - they'll be available to Minikube:

```bash
bash infrastructure/k8s/scripts/build-images.sh
```

The script will use Podman (as you've already configured) and the images will be available to Kubernetes.

### 4. Deploy to Kubernetes

```bash
bash infrastructure/k8s/scripts/deploy.sh
```

### 5. Set Up Port Forwarding

```bash
bash infrastructure/k8s/scripts/port-forward.sh
```

That's it! Your services are now running.

## Alternative Approach: Local Registry

If you prefer not to use Minikube or want more control, you can use a local container registry.

### 1. Start a Local Registry

```bash
# Run a local registry with Podman
podman run -d -p 5000:5000 --name local-registry registry:2

# Verify it's running
curl http://localhost:5000/v2/_catalog
```

### 2. Build and Push Images

```bash
# Build images
bash infrastructure/k8s/scripts/build-images.sh

# Tag for local registry
podman tag ecommerce-auth-service:latest localhost:5000/ecommerce-auth-service:latest
podman tag ecommerce-product-service:latest localhost:5000/ecommerce-product-service:latest
podman tag ecommerce-order-service:latest localhost:5000/ecommerce-order-service:latest

# Push to local registry
podman push localhost:5000/ecommerce-auth-service:latest
podman push localhost:5000/ecommerce-product-service:latest
podman push localhost:5000/ecommerce-order-service:latest
```

### 3. Update Kubernetes Manifests

You'll need to update the image references in the service manifests to point to your local registry:

```bash
# Use the helper script
bash infrastructure/k8s/scripts/podman-update-manifests.sh
```

Or manually edit each service YAML:
- `infrastructure/k8s/services/auth-service.yaml`
- `infrastructure/k8s/services/product-service.yaml`
- `infrastructure/k8s/services/order-service.yaml`

Change:
```yaml
image: ecommerce-auth-service:latest
imagePullPolicy: Never
```

To:
```yaml
image: localhost:5000/ecommerce-auth-service:latest
imagePullPolicy: Always
```

### 4. Configure Kubernetes to Use Insecure Registry

For Minikube:
```bash
minikube start --insecure-registry="localhost:5000"
```

For kind, create a config file:
```yaml
# kind-config.yaml
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
containerdConfigPatches:
- |-
  [plugins."io.containerd.grpc.v1.cri".registry.mirrors."localhost:5000"]
    endpoint = ["http://localhost:5000"]
```

Then create the cluster:
```bash
kind create cluster --name ecommerce --config=kind-config.yaml
```

### 5. Deploy

```bash
bash infrastructure/k8s/scripts/deploy.sh
bash infrastructure/k8s/scripts/port-forward.sh
```

## Troubleshooting

### Images Not Found

If you see `ImagePullBackOff` or `ErrImagePull`:

```bash
# Check if minikube can see the images
eval $(minikube podman-env)
podman images | grep ecommerce

# If empty, rebuild
bash infrastructure/k8s/scripts/build-images.sh
```

### Podman Machine Issues

```bash
# Check machine status
podman machine ls

# Restart machine if needed
podman machine stop
podman machine start

# Reset minikube
minikube delete
minikube start --driver=podman
```

### Port Conflicts

Podman runs in a VM on macOS, so port forwarding works slightly differently:

```bash
# Check podman machine ports
podman machine inspect

# Ensure ports are accessible
kubectl port-forward --address 0.0.0.0 svc/auth-service 50051:50051 -n ecommerce
```

## Development Workflow

### Rebuild After Code Changes

```bash
# Make sure you're using minikube's environment
eval $(minikube podman-env)

# Rebuild specific service
podman build -t ecommerce-auth-service:latest -f services/auth-service/Dockerfile .

# Restart the deployment
kubectl rollout restart deployment/auth-service -n ecommerce
```

### Quick Rebuild Script

```bash
#!/bin/bash
# rebuild-service.sh <service-name>

SERVICE=$1
eval $(minikube podman-env)
podman build -t ecommerce-${SERVICE}:latest -f services/${SERVICE}/Dockerfile .
kubectl rollout restart deployment/${SERVICE} -n ecommerce
```

Usage:
```bash
bash rebuild-service.sh auth-service
```

## Podman vs Docker Differences

| Aspect | Docker | Podman |
|--------|--------|--------|
| Daemon | Requires daemon | Daemonless |
| Root | Usually requires root | Rootless by default |
| Kubernetes Integration | Direct | Via Minikube or registry |
| Port Mapping | Direct to host | Via VM on macOS |
| Compose | docker-compose | podman-compose or podman kube play |

## Using Podman's Kubernetes Features

Podman has built-in Kubernetes YAML generation:

```bash
# Generate Kubernetes YAML from running container
podman generate kube <container-name> > service.yaml

# Play Kubernetes YAML with Podman
podman kube play infrastructure/k8s/services/auth-service.yaml
```

However, for full microservices testing, using Minikube is recommended as it provides:
- Better multi-service orchestration
- Service discovery
- Proper networking
- Resource management
- Closer to production Kubernetes

## Clean Up

```bash
# Stop port forwarding
pkill -f "kubectl port-forward"

# Delete Kubernetes resources
bash infrastructure/k8s/scripts/cleanup.sh

# Stop minikube
minikube stop

# Optional: Delete minikube cluster
minikube delete

# Optional: Stop and remove local registry
podman stop local-registry
podman rm local-registry
```

## Summary

**Recommended for Podman users:**
1. Use Minikube with Podman driver
2. Use `eval $(minikube podman-env)` before building
3. Build images with the provided script
4. Deploy normally with kubectl

This gives you the best experience with Podman while getting full Kubernetes functionality for testing.
