use actix_web::{get, Responder, Result};
use tracing::{info, instrument};

#[get("/")]
#[instrument(name = "http_index_handler")]
pub async fn index() -> Result<impl Responder> {
    info!("Processing request in index handler");
    Ok("Hello from Actix + OpenTelemetry!")
}
