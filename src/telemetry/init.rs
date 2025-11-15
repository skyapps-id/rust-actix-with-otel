use chrono::Utc;
use opentelemetry::{global, trace::TraceContextExt};
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{ExportConfig, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing::{Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn init_tracer() {
    let export_config = ExportConfig {
        endpoint: Some("http://127.0.0.1:4317".to_string()),
        ..Default::default()
    };

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_export_config(export_config)
        .build()
        .expect("failed to build OTLP exporter");

    let resource = Resource::builder()
        .with_attributes(vec![KeyValue::new("service.name", "my_actix_service")])
        .build();

    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer("actix_tracer");
    global::set_tracer_provider(provider);

    let otel_layer = tracing_opentelemetry::layer::<Registry>().with_tracer(tracer);

    let fmt_layer = fmt::layer().event_format(CustomJson);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt_layer)
        .init();
}

struct CustomJson;

impl<S, N> fmt::FormatEvent<S, N> for CustomJson
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        let span = Span::current();
        let ctx_otel = span.context();
        let span_ref = ctx_otel.span();
        let span_ctx = span_ref.span_context();
        let trace_id = span_ctx.trace_id();
        let span_id = span_ctx.span_id();

        let mut msg_buf = String::new();
        let formatter = fmt::format().with_target(false).without_time();
        let writer_buf = fmt::format::Writer::new(&mut msg_buf);
        formatter.format_event(ctx, writer_buf, event)?;
        let message = msg_buf.trim();

        let span_name = if let Some(scope) = ctx.lookup_current() {
            scope.metadata().name().to_string()
        } else {
            "".to_string()
        };

        write!(writer, "{{")?;
        write!(writer, "\"timestamp\":\"{}\"", Utc::now().to_rfc3339())?;
        write!(writer, ",\"level\":\"{}\"", meta.level())?;

        if trace_id != opentelemetry::trace::TraceId::INVALID {
            write!(writer, ",\"trace_id\":\"{}\"", trace_id)?;
        }
        if span_id != opentelemetry::trace::SpanId::INVALID {
            write!(writer, ",\"span_id\":\"{}\"", span_id)?;
        }

        write!(writer, ",\"fields\":{{\"message\":\"{}\"}}", message)?;
        write!(writer, ",\"span\":{{\"name\":\"{}\"}}", span_name)?;
        write!(writer, "}}\n")?;

        Ok(())
    }
}
