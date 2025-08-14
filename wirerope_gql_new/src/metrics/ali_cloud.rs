#![allow(unused)] // 允许整个文件中的未使用代码

use actix_web_opentelemetry::PrometheusMetricsHandler;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{Aggregation, Instrument, SdkMeterProvider, Stream},
    propagation::TraceContextPropagator,
    trace::SdkTracerProvider,
    Resource,
};
use tonic::metadata::MetadataMap;

use crate::config::Metrics;

static SLS_PROJECT_HEADER: &str = "x-sls-otel-project";
static SLS_INSTANCE_ID_HEADER: &str = "x-sls-otel-instance-id";
static SLS_AK_ID_HEADER: &str = "x-sls-otel-ak-id";
static SLS_AK_SECRET_HEADER: &str = "x-sls-otel-ak-secret";

static SLS_SERVICE_VERSION: &str = "service.version";
static SLS_SERVICE_NAME: &str = "service.name";
static SLS_SERVICE_NAMESPACE: &str = "service.namespace";
static SLS_HOST_NAME: &str = "host.name";

///
/// 初始化aliyun 需要的指标参数
fn load_aliyun_sls_env() -> (MetadataMap, Vec<KeyValue>) {
    let mut metadata_map = MetadataMap::with_capacity(4);
    metadata_map.insert(
        SLS_PROJECT_HEADER,
        option_env!("METRIC_SLS_PROJECT")
            .unwrap_or_default()
            .parse()
            .expect("SLS Params <SLS_PROJECT> failed!"),
    );
    metadata_map.insert(
        SLS_INSTANCE_ID_HEADER,
        option_env!("METRIC_INSTANCE_ID")
            .unwrap_or_default()
            .parse()
            .expect("SLS Params <INSTANCE_ID> failed!"),
    );
    metadata_map.insert(
        SLS_AK_ID_HEADER,
        option_env!("METRIC_AK_ID")
            .unwrap_or_default()
            .parse()
            .expect("SLS Params <AK_ID> failed!"),
    );
    metadata_map.insert(
        SLS_AK_SECRET_HEADER,
        option_env!("METRIC_AK_SECRET")
            .unwrap_or_default()
            .parse()
            .expect("SLS Params <AK_SECRET> failed!"),
    );

    let resource = vec![
        KeyValue::new(SLS_SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        KeyValue::new(
            SLS_HOST_NAME,
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "prod"
            },
        ),
        KeyValue::new(
            SLS_SERVICE_NAME,
            option_env!("SERVICE_NAME").unwrap_or("unknown-service-name"),
        ),
    ];
    (metadata_map, resource)
}

///
/// 指标监控 trace初始化
pub fn setup_metrics_tracing(service_name: &str, metrics: &Metrics) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Setup metrics of aliyun cloud sls...");
    tracing::debug!("Metrics config: {:?}", metrics);

    let exporter = MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create metric exporter");

    
    global::set_text_map_propagator(TraceContextPropagator::new());
    let (meta_map, resources) = load_aliyun_sls_env();
    let tracer = SdkTracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint( &metrics.end_point)
                .build()?,
        )
        .with_resource(Resource::builder_empty().with_attributes(resources).build())
        .build();
    global::set_tracer_provider(tracer.clone());
    tracing::info!("Metrics done!");
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
