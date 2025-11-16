mod handlers;
mod models;
mod services;
mod telemetry;

use actix_web::{App, HttpServer};
use tracing_actix_web::TracingLogger;
use opentelemetry_instrumentation_actix_web::RequestTracing;
use handlers::{index::index, todo::todo_handler};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    telemetry::init::init_tracer();

    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(RequestTracing::new())
            .service(index)
            .service(todo_handler)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
