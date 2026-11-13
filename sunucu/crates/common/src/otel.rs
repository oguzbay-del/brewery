//! OpenTelemetry tracing (feature `otel`, ADR-0008 rule 11.3).

use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{Error, Result};

/// Environment variable that switches OTLP trace export on (standard OpenTelemetry name).
pub const ENV_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// Keeps the tracer provider alive; flushes pending spans when dropped.
#[derive(Debug)]
pub struct TelemetryGuard {
    provider: SdkTracerProvider,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Err(e) = self.provider.shutdown() {
            eprintln!("telemetry shutdown failed: {e}");
        }
    }
}

/// Like [`crate::telemetry::init_tracing`], plus an OpenTelemetry layer.
///
/// A tracer provider is always installed so spans carry valid trace ids (logs and the W3C
/// `traceparent` header work); spans are exported over OTLP/HTTP only when
/// `OTEL_EXPORTER_OTLP_ENDPOINT` is set. Keep the returned guard alive for the life of the
/// process.
pub fn init_tracing_otel(service_name: &str) -> Result<TelemetryGuard> {
    let export = std::env::var(ENV_OTLP_ENDPOINT).is_ok_and(|v| !v.trim().is_empty());
    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", service_name.to_owned()))
        .build();
    let mut builder = SdkTracerProvider::builder().with_resource(resource);
    if export {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()
            .map_err(|e| Error::Telemetry(e.to_string()))?;
        builder = builder.with_batch_exporter(exporter);
    }
    let provider = builder.build();
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = provider.tracer(service_name.to_owned());

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json().flatten_event(true))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .map_err(|e| Error::Telemetry(e.to_string()))?;
    tracing::info!(
        service = service_name,
        otlp_export = export,
        "tracing initialised"
    );
    Ok(TelemetryGuard { provider })
}
