use super::{
    model::{Project, ProjectCreate},
    repo,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetAllError {
    #[error("repo failed")]
    RepoError(#[source] repo::GetAllError),
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("repo failed")]
    RepoError(#[source] repo::CreateError),
}

#[derive(Clone)]
pub struct ProjectService {
    repo: repo::ProjectRepo,
}

impl ProjectService {
    pub fn new(repo: repo::ProjectRepo) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<Project>, GetAllError> {
        let projects = self.repo.get_all().await.map_err(GetAllError::RepoError)?;

        Ok(projects)
    }

    pub async fn create(&self, project_create: ProjectCreate) -> Result<Project, CreateError> {
        // TODO: Validate project create

        let project = self
            .repo
            .create(project_create)
            .await
            .map_err(CreateError::RepoError)?;

        Ok(project)
    }
}
