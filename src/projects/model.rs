use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, FromRow, PartialEq, Eq)]
pub struct Project {
    pub id: u64,
    // TODO: This probably needs to be a u64. Change in SQL table schema reauired too as SERIAL is not a regular 32 bit int.
    pub user_id: i32,
    pub name: String,
}

// TODO: Make proper validation.
#[derive(Deserialize, Validate)]
pub struct ProjectCreate {
    #[validate(length(min = 1))]
    pub name: String,
    pub source: String,
}

#[derive(Deserialize, Validate)]
pub struct ProjectUpdate {
    pub source: String,
}
