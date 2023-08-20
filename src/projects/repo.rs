use super::model::{Project, ProjectCreate, ProjectUpdate};
use async_trait::async_trait;
use sqlx::MySqlPool;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProjectRepoExt {
    /// Finds single project by id.
    /// If project with given id does not exist returns error `RepoError::NotFound`.
    async fn find_for_user(
        &self,
        user_id: i32,
        project_id: u64,
    ) -> Result<Option<Project>, RepoError>;

    /// Finds all projects for specified user.
    async fn find_all_for_user(&self, user_id: i32) -> Result<Vec<Project>, RepoError>;

    /// Checks if there is a project with given id that belongs to the given user.
    async fn exists(&self, user_id: i32, project_id: u64) -> Result<bool, RepoError>;

    /// Creates a new project in the database.
    async fn create(
        &self,
        user_id: i32,
        project_create: ProjectCreate,
    ) -> Result<Project, RepoError>;

    /// Updates user's project by id.
    async fn update(
        &self,
        user_id: i32,
        project_id: u64,
        project_update: ProjectUpdate,
    ) -> Result<(), RepoError>;
}

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("failed to query")]
    QueryError(#[source] sqlx::Error),
}

/// `ProjectRepo` provides functionality for managing projects in the database.
#[derive(Clone)]
pub struct ProjectRepo {
    db: MySqlPool,
}

impl ProjectRepo {
    pub fn new(db: MySqlPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectRepoExt for ProjectRepo {
    async fn find_for_user(
        &self,
        user_id: i32,
        project_id: u64,
    ) -> Result<Option<Project>, RepoError> {
        let sql = "SELECT * FROM projects WHERE id = ? AND user_id = ? LIMIT 1";

        Ok(sqlx::query_as::<_, Project>(sql)
            .bind(project_id)
            .bind(user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(RepoError::QueryError)?)
    }

    async fn find_all_for_user(&self, user_id: i32) -> Result<Vec<Project>, RepoError> {
        let sql = "SELECT * FROM projects WHERE user_id = ?";

        Ok(sqlx::query_as::<_, Project>(sql)
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .map_err(RepoError::QueryError)?)
    }

    async fn exists(&self, user_id: i32, project_id: u64) -> Result<bool, RepoError> {
        let sql = "SELECT 1 FROM projects WHERE id = ? AND user_id = ? LIMIT 1";

        Ok(sqlx::query(sql)
            .bind(project_id)
            .bind(user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(RepoError::QueryError)?
            .is_some())
    }

    async fn create(
        &self,
        user_id: i32,
        ProjectCreate { name, source }: ProjectCreate,
    ) -> Result<Project, RepoError> {
        let mut tx = self.db.begin().await.map_err(RepoError::QueryError)?;

        let sql = "INSERT INTO projects (user_id, name, source) VALUES (?, ?, ?)";

        sqlx::query(sql)
            .bind(user_id)
            .bind(name)
            .bind(source)
            .execute(&mut *tx)
            .await
            .map_err(RepoError::QueryError)?;

        let sql = "SELECT * FROM projects WHERE id = LAST_INSERT_ID()";

        let project = sqlx::query_as::<_, Project>(sql)
            .fetch_one(&mut *tx)
            .await
            .map_err(RepoError::QueryError)?;

        tx.commit().await.map_err(RepoError::QueryError)?;

        Ok(project)
    }

    async fn update(
        &self,
        user_id: i32,
        project_id: u64,
        ProjectUpdate { source }: ProjectUpdate,
    ) -> Result<(), RepoError> {
        let sql = "UPDATE projects SET source = ? WHERE id = ? AND user_id = ?";

        sqlx::query(sql)
            .bind(source)
            .bind(project_id)
            .bind(user_id)
            .execute(&self.db)
            .await
            .map_err(RepoError::QueryError)?;

        Ok(())
    }
}
