use playground_api::{config, startup, telemetry};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Load local environment overrides before any subsystems boot.
    // In production environments, this silently fails and defers to system variables.
    dotenvy::dotenv().ok();

    telemetry::init_subscriber();

    // Enforce configuration contracts immediately to prevent partial boot states.
    let config = match config::AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Fatal configuration error: {}", e);
            return Err(e);
        }
    };

    let (listener, router) = startup::build_application(config.clone()).await?;

    info!(
        host = %config.server.host,
        port = %config.server.port,
        "Server is ready and listening"
    );

    axum::serve(listener, router).await.map_err(|e| {
        error!("Server crashed: {}", e);
        e.to_string()
    })?;

    Ok(())
}
