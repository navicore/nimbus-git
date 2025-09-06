.PHONY: help build test deploy clean setup-k8s

help: ## Show this help message
	@echo "Nimbus Git Platform - Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-15s %s\n", $$1, $$2}'

build: ## Build the Nimbus Docker image
	@echo "Building Nimbus image..."
	podman build -t localhost/nimbus-git:latest .

test: ## Run tests
	@echo "Running tests..."
	cargo test --all

lint: ## Run linters
	@echo "Running cargo fmt check..."
	cargo fmt --all -- --check
	@echo "Running cargo clippy..."
	cargo clippy --all -- -D warnings

setup-k8s: ## Setup Kubernetes cluster with Kind
	@echo "Setting up Kind cluster..."
	kind create cluster --config k8s/kind-config.yaml || true
	@echo "Installing nginx ingress..."
	kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml
	@echo "Waiting for ingress to be ready..."
	kubectl wait --namespace ingress-nginx \
		--for=condition=ready pod \
		--selector=app.kubernetes.io/component=controller \
		--timeout=90s || true

deploy: build ## Deploy Nimbus to Kubernetes
	@echo "Loading image into Kind..."
	@# Save and load image
	podman save -o /tmp/nimbus-git.tar localhost/nimbus-git:latest
	DOCKER_HOST="unix://$$HOME/.local/share/containers/podman/machine/podman.sock" \
		kind load image-archive /tmp/nimbus-git.tar || \
		echo "Note: If using podman, image will be pulled from localhost registry"
	@echo "Creating namespace..."
	kubectl create namespace nimbus || true
	@echo "Applying Kubernetes manifests..."
	kubectl apply -f k8s/owner-secret.yaml
	kubectl apply -f k8s/pvc.yaml
	kubectl apply -f k8s/deployment.yaml
	kubectl apply -f k8s/service.yaml
	kubectl apply -f k8s/ingress.yaml || true
	@echo "Deployment complete!"
	@echo ""
	@echo "Nimbus is deploying. Check status with: kubectl get pods -n nimbus"
	@echo "Access at: https://code.navicore.tech (if cloudflared is configured)"

init-password: ## Initialize admin password (usage: make init-password PASSWORD=yourpassword)
	@if [ -z "$(PASSWORD)" ]; then \
		echo "Error: PASSWORD not set. Usage: make init-password PASSWORD=yourpassword"; \
		exit 1; \
	fi
	@./scripts/init-password.sh "$(PASSWORD)"

logs: ## Show Nimbus logs
	kubectl logs -n nimbus -l app=nimbus --tail=50 -f

status: ## Show deployment status
	@echo "=== Nimbus Deployment Status ==="
	@echo ""
	@echo "Pods:"
	@kubectl get pods -n nimbus
	@echo ""
	@echo "Services:"
	@kubectl get svc -n nimbus
	@echo ""
	@echo "Secrets:"
	@kubectl get secrets -n nimbus | grep nimbus
	@echo ""
	@echo "Health check:"
	@curl -s https://code.navicore.tech/health 2>/dev/null | jq . || echo "Not accessible via tunnel"

clean: ## Clean up Kubernetes resources
	@echo "Cleaning up Nimbus deployment..."
	kubectl delete namespace nimbus || true
	@echo "Cleanup complete"

restart: ## Restart Nimbus deployment
	kubectl rollout restart deployment/nimbus-web -n nimbus
	kubectl rollout status deployment/nimbus-web -n nimbus

port-forward: ## Setup local port forwarding for development
	@echo "Setting up port forwarding..."
	@echo "Nimbus will be available at http://localhost:3002"
	kubectl port-forward -n nimbus svc/nimbus-web 3002:3000

dev: ## Run development server locally
	@echo "Starting development server..."
	RUST_LOG=debug cargo run --bin nimbus-web