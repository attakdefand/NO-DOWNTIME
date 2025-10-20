mod server;
mod health;
mod config;
mod auth;
mod tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with OpenTelemetry
    no_downtime_service::tracing::init_tracing()?;

    // Load configuration
    let config = config::Config::from_env()?;

    // Start the server
    server::run(config).await?;

    // Shutdown tracing gracefully
    no_downtime_service::tracing::shutdown_tracing();
    
    Ok(())
}