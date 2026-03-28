//! Observability and logging initialization.
//!
//! This module configures a multi-layered tracing subscriber:
//! 1. A JSON formatting layer for standard output (scraped by Alloy/Loki).
//! 2. An optional OpenTelemetry (OTLP) layer for distributed tracing (sent to Tempo).

use opentelemetry::global;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

/// Configures global tracing subscribers and optional OpenTelemetry exporters.
///
/// If OTEL_EXPORTER_OTLP_ENDPOINT is present in the environment, it initializes
/// a gRPC OTLP pipeline for distributed tracing.
pub fn init_subscriber() {
    // Determine log verbosity from RUST_LOG or LOG_LEVEL environment variables.
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            EnvFilter::try_new(std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()))
        })
        .unwrap();

    // Standard JSON formatter. 'flatten_event' ensures fields are at the root
    // of the JSON object, which simplifies LogQL queries in Grafana/Loki.
    let formatting_layer = tracing_subscriber::fmt::layer().json().flatten_event(true);

    // Build the optional OpenTelemetry layer if an endpoint is provided.
    let telemetry_layer = if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        // Initialize the OTLP exporter using the tonic (gRPC) runtime.
        let exporter = SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to create OTLP span exporter");

        // Create a resource describing the application.
        let resource = Resource::builder()
            .with_service_name("playground-api")
            .build();

        // Create the provider which manages the span lifecycle and batching.
        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();

        // Set this provider as the global default for the application.
        global::set_tracer_provider(provider.clone());

        // Create a tracer specifically for our application boundary.
        let tracer = global::tracer("playground-api");

        Some(tracing_opentelemetry::layer().with_tracer(tracer))
    } else {
        None
    };

    // Initialize the global tracing registry with our composed layers.
    Registry::default()
        .with(filter)
        .with(formatting_layer)
        .with(telemetry_layer)
        .init();
}
