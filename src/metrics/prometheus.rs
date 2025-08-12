use actix_web_opentelemetry::PrometheusMetricsHandler;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{Aggregation, Instrument, SdkMeterProvider, Stream},
    propagation::TraceContextPropagator,
    trace::SdkTracerProvider,
    Resource,
};
use crate::config::Metrics;


///
/// 指标监控 trace初始化
#[allow(unused)]
pub fn setup_metrics_tracing(
    service_name: &str,
    metrics: &Metrics
) -> Result<(), Box<dyn std::error::Error>> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let service_name_resource = Resource::builder_empty()
        .with_attribute(KeyValue::new("service.name", service_name.to_owned()))
        .build();
    let tracer = SdkTracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(metrics.end_point.clone())
                .build()?,
        )
        .with_resource(service_name_resource)
        .build();
    global::set_tracer_provider(tracer.clone());
    Ok(())
}

///
/// 指标 prometheus接口响应
#[allow(unused)]
pub fn gen_prometheus_handler(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // prometheus
    let (metrics_handler, meter_provider) = {
        let registry = prometheus::Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;

        let provider = SdkMeterProvider::builder()
            .with_reader(exporter)
            .with_resource(
                Resource::builder_empty()
                    .with_attribute(KeyValue::new("service.name", service_name.to_owned()))
                    .build(),
            )
            .with_view(
                opentelemetry_sdk::metrics::new_view(
                    Instrument::new().name("http.server.duration"),
                    Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                        boundaries: vec![
                            0.0, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5,
                            5.0, 7.5, 10.0,
                        ],
                        record_min_max: true,
                    }),
                )
                .unwrap(),
            )
            .build();
        global::set_meter_provider(provider.clone());

        (PrometheusMetricsHandler::new(registry), provider)
    };
    tracing::info!("init metrics done!");
    Ok(())
}
