use std::sync::Arc;

use super::{
    model::{Project, ProjectCreate, ProjectUpdate},
    repo::{self, ProjectRepoExt},
};
use anyhow::anyhow;
use async_trait::async_trait;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProjectServiceExt {
    /// Finds all projects for the given user.
    async fn find_all_for_user(&self, user_id: i32) -> Result<Vec<Project>, ServiceError>;

    /// Finds a project with the given id for the given user.
    async fn find_for_user(
        &self,
        user_id: i32,
        project_id: u64,
    ) -> Result<Option<Project>, ServiceError>;

    /// Creates a new project for the given user.
    /// Validates `project_data` before inserting.
    async fn create(
        &self,
        user_id: i32,
        project_create: ProjectCreate,
    ) -> Result<Project, ServiceError>;

    /// Updates a project for the given user.
    async fn update(
        &self,
        user_id: i32,
        project_id: u64,
        project_update: ProjectUpdate,
    ) -> Result<(), ServiceError>;
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("repository failed")]
    RepoError(#[source] repo::RepoError),

    #[error("missing resource")]
    MissingResource(#[source] anyhow::Error),

    #[error("validation failed")]
    ValidationError(#[source] ValidationErrors),
}

#[derive(Clone)]
pub struct ProjectService {
    repo: Arc<dyn ProjectRepoExt + Send + Sync>,
}

impl ProjectService {
    pub fn new(repo: Arc<dyn ProjectRepoExt + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ProjectServiceExt for ProjectService {
    async fn find_all_for_user(&self, user_id: i32) -> Result<Vec<Project>, ServiceError> {
        let projects = self
            .repo
            .find_all_for_user(69)
            .await
            .map_err(ServiceError::RepoError)?;

        Ok(projects)
    }

    async fn find_for_user(
        &self,
        user_id: i32,
        project_id: u64,
    ) -> Result<Option<Project>, ServiceError> {
        let project = self
            .repo
            .find_for_user(user_id, project_id)
            .await
            .map_err(ServiceError::RepoError)?;

        Ok(project)
    }

    async fn create(
        &self,
        user_id: i32,
        project_create: ProjectCreate,
    ) -> Result<Project, ServiceError> {
        if let Err(e) = project_create.validate() {
            return Err(ServiceError::ValidationError(e));
        }

        let project = self
            .repo
            .create(user_id, project_create)
            .await
            .map_err(ServiceError::RepoError)?;

        Ok(project)
    }

    async fn update(
        &self,
        user_id: i32,
        project_id: u64,
        project_update: ProjectUpdate,
    ) -> Result<(), ServiceError> {
        let project_exits = self
            .repo
            .exists(user_id, project_id)
            .await
            .map_err(ServiceError::RepoError)?;

        if !project_exits {
            return Err(ServiceError::MissingResource(anyhow!(
                "project does not exist"
            )));
        }

        self.repo
            .update(user_id, project_id, project_update)
            .await
            .map_err(ServiceError::RepoError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projects::{model::Project, repo::MockProjectRepoExt};

    fn fake_project_query(project_id: u64, user_id: i32) -> Project {
        Project {
            id: project_id,
            user_id,
            name: "test_project".to_string(),
        }
    }

    fn fake_projects_query(user_id: i32) -> Vec<Project> {
        vec![
            fake_project_query(1, user_id),
            fake_project_query(2, user_id),
            fake_project_query(445, user_id),
        ]
    }

    #[tokio::test]
    async fn test_find_all_for_user() {
        // arrange
        let user_id = 1;
        let want = fake_projects_query(user_id);

        let mut project_repo = MockProjectRepoExt::new();
        project_repo
            .expect_find_all_for_user()
            .returning(|user_id| Ok(fake_projects_query(user_id)));

        // act
        let project_service = ProjectService::new(Arc::from(project_repo));

        // assert
        let got = project_service.find_all_for_user(user_id).await.unwrap();
        assert_eq!(got, want);
    }

    #[tokio::test]
    async fn test_find_for_user() {
        // arrange
        let project_id: u64 = 1;
        let user_id = 1;

        let want = Some(fake_project_query(project_id, user_id));

        let mut project_repo = MockProjectRepoExt::new();
        project_repo
            .expect_find_for_user()
            .returning(|user_id, project_id| Ok(Some(fake_project_query(project_id, user_id))));

        // act
        let project_service = ProjectService::new(Arc::from(project_repo));

        // assert
        let got = project_service
            .find_for_user(user_id, project_id)
            .await
            .unwrap();

        assert_eq!(got, want);
    }
}
