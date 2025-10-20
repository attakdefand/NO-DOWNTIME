use crate::{config::Config, health};
use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;

use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Create the application routes
    let app = create_app();

    // Bind to the address
    let addr: SocketAddr = config.bind_address;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

pub fn create_app() -> Router {
    Router::new()
        // Health check endpoints (essential for zero-downtime)
        .route("/live", get(health::live_handler))
        .route("/ready", get(health::ready_handler))
        // Application routes would go here
        .route("/", get(root_handler))
        // Middleware for observability
        .layer(TraceLayer::new_for_http())
        // Add CORS for web compatibility
        .layer(CorsLayer::very_permissive())
}

async fn root_handler() -> &'static str {
    "Hello, Zero-Downtime World!"
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown");
    
    // Mark service as not ready during shutdown
    health::set_not_ready();
    
    // Allow time for load balancers to notice we're not ready
    tokio::time::sleep(Duration::from_secs(5)).await;
}