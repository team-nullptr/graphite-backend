use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Project {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ProjectCreate {
    pub name: String,
}
