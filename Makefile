# Makefile for No-Downtime Service

# Build the application
build:
	cargo build

# Run the application
run:
	cargo run

# Run tests
test:
	cargo test -- --test-threads=1

# Run tests with verbose output
test-verbose:
	cargo test -- --test-threads=1 --nocapture

# Check for compilation warnings
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run linter
clippy:
	cargo clippy

# Build Docker image
docker-build:
	docker build -t no-downtime-service .

# Apply Kubernetes manifests
k8s-apply:
	kubectl apply -f k8s/

# Delete Kubernetes resources
k8s-delete:
	kubectl delete -f k8s/

# Clean target directory
clean:
	cargo clean

.PHONY: build run test test-verbose check fmt clippy docker-build k8s-apply k8s-delete clean