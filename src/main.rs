use actix_web::{get, App, HttpServer, Responder};
use chrono::Utc;
use opentelemetry::global;
use opentelemetry::trace::{TraceContextExt, TracerProvider};
use opentelemetry::KeyValue;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing::{info, instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{
    fmt::{self, format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

#[get("/")]
#[instrument(name = "http_index_handler")]
async fn index() -> impl Responder {
    info!("Processing request in index handler");
    "Hello from Actix + OpenTelemetry!"
}

fn init_tracer() {
    let exporter = SpanExporter::builder()
        .with_tonic()
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
    // let fmt_layer = fmt::layer().json();

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt_layer)
        .init();
}

struct CustomJson;

impl<S, N> FormatEvent<S, N> for CustomJson
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        // Ambil trace_id & span_id
        let span = Span::current();
        let ctx_otel = span.context();
        let span_ref = ctx_otel.span();
        let span_ctx = span_ref.span_context();

        let trace_id = span_ctx.trace_id();
        let span_id = span_ctx.span_id();

        let mut msg_buf = String::new();
        let formatter = fmt::format().with_target(false);
        let writer_buf = fmt::format::Writer::new(&mut msg_buf);
        formatter.format_event(ctx, writer_buf, event)?;

        write!(writer, "{{")?;
        write!(writer, "\"timestamp\":\"{}\"", Utc::now().to_rfc3339())?;
        write!(writer, ",\"level\":\"{}\"", meta.level())?;

        if trace_id != opentelemetry::trace::TraceId::INVALID {
            write!(writer, ",\"trace_id\":\"{}\"", trace_id)?;
        }
        if span_id != opentelemetry::trace::SpanId::INVALID {
            write!(writer, ",\"span_id\":\"{}\"", span_id)?;
        }

        write!(writer, ",\"message\":\"{}\"", msg_buf.trim())?;
        write!(writer, "}}\n")?;

        Ok(())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracer();

    HttpServer::new(|| App::new().service(index))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
