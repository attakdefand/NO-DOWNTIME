# No-Downtime Service

A Rust-based web service implementing zero-downtime deployment principles based on the NO-DOWNTIME.MD guidelines.

## Documentation

- [NO-DOWNTIME.MD](NO-DOWNTIME.MD) - Core zero-downtime principles
- [PRODUCT-FEATURES.MD](PRODUCT-FEATURES.MD) - Current feature implementation status
- [PRODUCT-ROADMAP.MD](PRODUCT-ROADMAP.MD) - Development roadmap
- [SECURITY-LAYER.MD](SECURITY-LAYER.MD) - Security implementation details
- [IMPLEMENTATION_SUMMARY.MD](IMPLEMENTATION_SUMMARY.MD) - Technical implementation details

## Features

This service implements the following zero-downtime principles:

1. **Health Checks**: Liveness and readiness probes for Kubernetes integration
2. **Graceful Shutdown**: Proper handling of SIGTERM with pre-stop hooks
3. **Timeouts**: Request timeouts to prevent resource exhaustion
4. **Backpressure**: Concurrency limits to prevent overload
5. **Observability**: Tracing and logging for monitoring
6. **Containerization**: Docker support for consistent deployments

See [PRODUCT-FEATURES.MD](PRODUCT-FEATURES.MD) for a complete list of features and their implementation status.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Docker (optional, for containerization)
- Kubernetes (optional, for orchestration)

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run
```

Or with custom configuration:

```bash
APP_BIND_ADDRESS=0.0.0.0:3000 cargo run
```

### Testing

```bash
cargo test
```

## Zero-Downtime Deployment

This service is designed for zero-downtime deployment with:

- Kubernetes readiness/liveness probes
- Graceful shutdown handling
- Rolling update strategy with maxUnavailable=0
- Pod disruption budgets
- Anti-affinity rules for high availability

## Configuration

The service can be configured using environment variables:

- `APP_BIND_ADDRESS`: The address to bind to (default: 0.0.0.0:3000)
- `APP_SHUTDOWN_TIMEOUT`: Graceful shutdown timeout in seconds (default: 30)

## Kubernetes Deployment

Apply the Kubernetes manifests:

```bash
kubectl apply -f k8s/
```

This will deploy:

- A deployment with rolling update strategy
- A service for internal communication
- A pod disruption budget for controlled updates

## Health Checks

The service exposes two health check endpoints:

- `/live`: Liveness probe (critical system health)
- `/ready`: Readiness probe (application readiness for traffic)

Both return JSON responses with status information.# NO-DOWNTIME
