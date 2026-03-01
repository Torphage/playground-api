use playground_api::{config, startup, telemetry};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Telemetry bootstrap comes first, so we can see config-loading failures.
    // Use a conservative default that works well in production.
    let log_handle = telemetry::init_subscriber("info");

    // Load config
    let config = config::Config::load().expect("Failed to read configuration.");

    // Apply config log level (unless RUST_LOG is set).
    telemetry::reload_filter(&log_handle, &config.log_level);

    // Wire everything up
    let (listener, router) = startup::build_application(config).await?;

    // Start serving
    axum::serve(listener, router)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
