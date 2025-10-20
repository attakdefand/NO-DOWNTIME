# No-Downtime Service Web UI

A professional web interface for monitoring and managing the No-Downtime Service, built with Rust, WebAssembly, and Yew.

## Features

- **Dashboard Overview**: Real-time status of the service with key metrics
- **Health Monitoring**: Visual indicators for liveness and readiness probes
- **Metrics Visualization**: Charts and data tables for HTTP requests, errors, and performance
- **Authentication Management**: OAuth2 integration and RBAC configuration
- **Service Configuration**: Manage service settings and TLS configuration
- **Real-time Monitoring**: Live updates and notifications

## Technologies Used

- **Rust**: Core programming language
- **Yew**: Frontend framework for building web applications with Rust
- **WebAssembly (WASM)**: Compilation target for running Rust in the browser
- **wasm-bindgen**: Binding generator for communicating between Rust and JavaScript
- **gloo**: Collection of crates for building WASM applications
- **serde**: Serialization framework

## UI Components

The web UI contains 45 professionally designed UI elements across 6 main sections:

1. **Dashboard Overview** (8 elements)
2. **Health Monitoring** (6 elements)
3. **Metrics Visualization** (10 elements)
4. **Authentication Panel** (6 elements)
5. **Configuration Panel** (8 elements)
6. **Real-time Monitoring** (7 elements)

## Project Structure

```
web-ui/
├── Cargo.toml                 # Web UI crate configuration
├── index.html                 # Main HTML file
├── assets/                    # Static assets
│   ├── css/                   # Stylesheets
│   │   └── styles.css         # Main stylesheet
│   ├── images/                # Image assets
│   └── favicon.ico            # Favicon
├── src/                       # Rust source code
│   ├── main.rs                # Entry point
│   ├── app.rs                 # Main application component
│   ├── components/            # UI components
│   │   ├── dashboard.rs       # Dashboard overview component
│   │   ├── health_monitor.rs  # Health monitoring component
│   │   ├── metrics_view.rs    # Metrics visualization component
│   │   ├── auth_panel.rs      # Authentication component
│   │   ├── config_panel.rs    # Configuration component
│   │   └── monitoring.rs      # Real-time monitoring component
│   ├── services/              # API services
│   │   ├── health.rs          # Health API service
│   │   ├── metrics.rs         # Metrics API service
│   │   ├── auth.rs            # Authentication service
│   │   └── config.rs          # Configuration service
│   └── utils/                 # Utility functions
│       ├── formatting.rs      # Data formatting utilities
│       └── http.rs            # HTTP utility functions
├── tests/                     # Frontend tests
│   ├── app_tests.rs           # Application tests
│   └── component_tests.rs     # Component tests
└── pkg/                       # Generated WASM package (created during build)
```

## Building and Running

To build and run the web UI, you'll need:

1. Rust and Cargo installed
2. wasm-pack installed (`cargo install wasm-pack`)
3. A static file server

### Development

```bash
# Install dependencies
wasm-pack build --target web

# Serve the application
# (Use any static file server, e.g., Python's built-in server)
python -m http.server 8000
```

### Production

```bash
# Build for production
wasm-pack build --target web --release
```

## Integration with No-Downtime Service

The web UI communicates with the No-Downtime Service through its REST API endpoints:

- `/live` - Liveness probe
- `/ready` - Readiness probe
- `/metrics` - Prometheus metrics (planned)
- `/oauth2/login` - OAuth2 authentication
- `/oauth2/callback` - OAuth2 callback
- `/protected` - Protected resources

## Professional Design Elements

- Responsive layout for different screen sizes
- Dark/light theme support
- Consistent color scheme with status indicators
- Interactive charts with tooltips
- Real-time data updates
- Role-based access control visualization
- Alert notifications system

## License

This project is part of the No-Downtime Service and is licensed under the same terms.