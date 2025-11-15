use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Todo {
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub id: u32,
    pub title: String,
    pub completed: bool,
}
