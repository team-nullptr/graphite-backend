use super::model::{Project, ProjectCreate};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("failed to fetch all projects")]
pub struct GetAllError(#[source] sqlx::Error);

#[derive(Error, Debug)]
#[error("failed to create a project")]
pub struct CreateError(#[source] sqlx::Error);

/// User repo gives an easy access to different user-related db operaions.
#[derive(Clone)]
pub struct ProjectRepo {
    db: sqlx::PgPool,
}

impl ProjectRepo {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    /// Retrieves all users from the database.
    pub async fn get_all(&self) -> Result<Vec<Project>, GetAllError> {
        let sql = "SELECT * FROM projects";

        Ok(sqlx::query_as::<_, Project>(sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| GetAllError(e))?)
    }

    /// Creates a new user in the database.
    pub async fn create(&self, project_create: ProjectCreate) -> Result<Project, CreateError> {
        let sql = "INSERT INTO projects (name) VALUES ($1) RETURNING *";

        Ok(sqlx::query_as::<_, Project>(sql)
            .bind(project_create.name)
            .fetch_one(&self.db)
            .await
            .map_err(|e| CreateError(e))?)
    }
}
