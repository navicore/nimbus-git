#!/bin/bash
# Deploy Nimbus to local Kubernetes (Kind/Podman)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying Nimbus Git Platform${NC}"

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v podman &> /dev/null; then
    echo -e "${RED}Error: Podman is not installed${NC}"
    exit 1
fi

if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}Error: kubectl is not installed${NC}"
    exit 1
fi

# Build the container image
echo -e "${YELLOW}Building container image...${NC}"
podman build -t nimbus-git:latest .
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Container image built${NC}"
else
    echo -e "${RED}✗ Failed to build container image${NC}"
    exit 1
fi

# Load image into Kind
echo -e "${YELLOW}Loading image into Kind cluster...${NC}"
podman save nimbus-git:latest | podman-machine ssh podman load
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Image loaded into Kind${NC}"
else
    # Try alternative method
    echo -e "${YELLOW}Trying alternative load method...${NC}"
    kind load docker-image nimbus-git:latest --name kind-cluster
fi

# Apply Kubernetes manifests
echo -e "${YELLOW}Applying Kubernetes manifests...${NC}"

# Create namespace
kubectl apply -f k8s/namespace.yaml
echo -e "${GREEN}✓ Namespace created${NC}"

# Create PVC
kubectl apply -f k8s/pvc.yaml
echo -e "${GREEN}✓ Persistent volume claim created${NC}"

# Create services
kubectl apply -f k8s/service.yaml
echo -e "${GREEN}✓ Services created${NC}"

# Deploy application
kubectl apply -f k8s/deployment.yaml
echo -e "${GREEN}✓ Deployment created${NC}"

# Create ingress (optional)
if kubectl get ingressclass nginx &> /dev/null; then
    kubectl apply -f k8s/ingress.yaml
    echo -e "${GREEN}✓ Ingress created${NC}"
else
    echo -e "${YELLOW}⚠ Nginx ingress controller not found, skipping ingress${NC}"
fi

# Wait for deployment
echo -e "${YELLOW}Waiting for deployment to be ready...${NC}"
kubectl rollout status deployment/nimbus-web -n nimbus --timeout=120s

# Get status
echo -e "${BLUE}Deployment status:${NC}"
kubectl get all -n nimbus

# Port forwarding info
echo -e "${GREEN}✅ Nimbus deployed successfully!${NC}"
echo -e "${BLUE}Access Nimbus:${NC}"
echo -e "  Web UI: kubectl port-forward -n nimbus svc/nimbus-web 3000:3000"
echo -e "  SSH: kubectl port-forward -n nimbus svc/nimbus-ssh 2222:22"
echo ""
echo -e "${BLUE}Or use NodePort for SSH:${NC}"
echo -e "  ssh git@localhost -p 30022"