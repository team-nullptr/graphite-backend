use crate::github;
use sqlx::PgPool;
use thiserror::Error;

use super::model::User;

#[derive(Error, Debug)]
#[error("failed to create a user")]
pub struct CreateError(#[source] sqlx::Error);

#[derive(Error, Debug)]
#[error("failed to find a user by id")]
pub struct FindByIdError(#[source] sqlx::Error);

#[derive(Clone)]
pub struct UserRepo {
    db: PgPool,
}

impl UserRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates internal user from github user.
    pub async fn create(&self, github_user: github::GitHubUser) -> Result<(), CreateError> {
        let sql = "INSERT INTO users (id, name, avatar_url) VALUES ($1, $2, $3)";

        sqlx::query(sql)
            .bind(github_user.id)
            .bind(github_user.name)
            .bind(github_user.avatar_url)
            .execute(&self.db)
            .await
            .map_err(|e| CreateError(e))?;

        Ok(())
    }

    /// Finds user by id.
    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>, FindByIdError> {
        let sql = "SELECT * FROM users WHERE id = $1 LIMIT 1";

        let user = sqlx::query_as::<_, User>(sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| FindByIdError(e))?;

        Ok(user)
    }
}
