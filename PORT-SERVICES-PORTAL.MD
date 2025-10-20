# Port, Services, and Portal Architecture

## Web UI Portals

The No-Downtime Service currently has **0 dedicated web UI portals**.

The service is a backend API service built with Rust and Axum framework that exposes RESTful endpoints but does not include any dedicated web user interface. Any web UI would need to be developed separately and would consume the API endpoints provided by this service.

## Services

The No-Downtime Service currently has **1 main web service** running on port 3000 (by default, configurable via environment variables).

### Service Endpoints

The service exposes the following HTTP endpoints:

1. `GET /` - Root endpoint returning a welcome message
2. `GET /live` - Liveness probe endpoint for health checking
3. `GET /ready` - Readiness probe endpoint for health checking
4. `GET /protected` - Example protected endpoint requiring authentication
5. `GET /oauth2/login` - OAuth2 authentication initiation endpoint
6. `GET /oauth2/callback` - OAuth2 authentication callback endpoint

### Service Features

The service includes several key features for enterprise deployment:
- Distributed tracing with OpenTelemetry
- Metrics collection with Prometheus
- OAuth2 authentication integration
- TLS support (configurable)
- Role-based access control (RBAC)
- Health checks for zero-downtime deployments
- Rate limiting and circuit breaking capabilities

## Terminal-Based Services

The No-Downtime Service currently has **0 dedicated terminal-based services**.

The service is a web-based API service that runs as a standard web server and does not expose any dedicated terminal interfaces or command-line interfaces. Management and interaction occur through the HTTP API endpoints.

## Port Configuration

The service listens on a single port (default: 3000) for all HTTP traffic. The port can be configured through the `BIND_ADDRESS` environment variable. When TLS is enabled, the same port handles both HTTP and HTTPS traffic with automatic detection.

## Architecture Summary

This is a single-service architecture providing a robust backend API with enterprise features suitable for zero-downtime deployments. The service is designed to be deployed behind a load balancer in production environments and can be scaled horizontally.