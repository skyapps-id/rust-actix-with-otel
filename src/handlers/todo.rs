use actix_web::{get, Responder};
use tracing::{info, instrument};
use crate::services::todo_service::fetch_todo;

#[get("/todo")]
#[instrument(name = "http_todo_handler")]
pub async fn todo_handler() -> impl Responder {
    match fetch_todo().await {
        Ok(todo) => {
            info!("Returning TODO result");
            format!(
                "TODO: id={} title={} completed={} user_id={}",
                todo.id, todo.title, todo.completed, todo.user_id,
            )
        }
        Err(err) => {
            info!("Error fetching remote API");
            format!("Error: {}", err)
        }
    }
}
