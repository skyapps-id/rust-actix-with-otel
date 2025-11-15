use tracing::{info, instrument};
use reqwest;
use crate::models::todo::Todo;

#[instrument(name = "fetch_todo_api", skip())]
pub async fn fetch_todo() -> Result<Todo, reqwest::Error> {
    info!("Fetching remote TODO");
    let client = reqwest::Client::new();

    let resp = client
        .get("https://jsonplaceholder.typicode.com/todos/1")
        .send()
        .await?
        .json::<Todo>()
        .await?;

    info!("Fetch complete");
    Ok(resp)
}