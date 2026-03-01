use tracing_subscriber::{
    layer::SubscriberExt,
    reload,
    util::SubscriberInitExt,
    EnvFilter, Registry
};

/// Handle allowing the app to adjust log filtering at runtime.
///
/// We *only* reload the filter (verbosity), not the formatting/output.
/// That keeps the logging pipeline stable and production-friendly.
pub type LogReloadHandle = reload::Handle<EnvFilter, Registry>;

/// Initialize telemetry **before configuration is loaded**.
///
/// Production-friendly defaults:
/// - Logs go to stderr (container/systemd friendly)
/// - JSON formatting (Loki/ELK friendly)
/// - Default filter = `info` unless `RUST_LOG` is set
///
/// Returns a reload handle that can be used after config is loaded to adjust
/// verbosity without rebuilding the entire subscriber.
pub fn init_subscriber(default_filter: &str) -> LogReloadHandle {
    // Filtering: prefer RUST_LOG, otherwise use a safe default.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    // Wrap the filter in a reloadable layer so we can update it later.
    let (filter_layer, handle) = reload::Layer::new(filter);

    // Formatting: we use JSON formatting. This is what Loki LOVES.
    // It includes timestamp, level, and all attached fields automatically.
    let formatting_layer = tracing_subscriber::fmt::layer()
        .json()
        .flatten_event(true); // Makes the JSON flatter and easier to query

    // Install global subscriber.
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(formatting_layer)
        .init();

    handle
}

/// Reload the log filter (verbosity) after configuration is loaded.
///
/// Behavior:
/// - If `RUST_LOG` is set, we do **not** override it (ops-friendly).
/// - Otherwise, we apply `new_filter` (e.g. `"debug"`, `"info,my_crate=trace"`, etc).
pub fn reload_filter(handle: &LogReloadHandle, new_filter: &str) {
    // Filtering: prefer RUST_LOG, otherwise use value from config.
    if std::env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
        tracing::debug!("RUST_LOG is set; skipping config-based log filter reload");
        return;
    }

    // EnvFilter accepts full directive syntax (e.g. "info,tower_http=debug").
    let filter = EnvFilter::new(new_filter);

    if let Err(err) = handle.reload(filter) {
        // Reload failures shouldn't crash the service. Log and continue.
        tracing::warn!(error = %err, "Failed to reload log filter; keeping previous filter");
    }
}
