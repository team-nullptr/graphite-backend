use super::repo::{self, UserRepo};
use crate::github;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateUserIfNewError {
    #[error("repo failed to find a user by id")]
    UserFindByIdError(#[source] repo::FindByIdError),

    #[error("repo failed to create a user")]
    UserCreateError(#[source] repo::CreateError),
}

#[derive(Clone)]
pub struct UserService {
    pub repo: UserRepo,
}

impl UserService {
    pub fn new(repo: UserRepo) -> Self {
        Self { repo }
    }

    pub async fn create_user_if_new(
        &self,
        github_user: github::GitHubUser,
    ) -> Result<(), CreateUserIfNewError> {
        let user = self
            .repo
            .find_by_id(github_user.id)
            .await
            .map_err(|e| CreateUserIfNewError::UserFindByIdError(e))?;

        if user.is_none() {
            self.repo
                .create(github_user)
                .await
                .map_err(|e| CreateUserIfNewError::UserCreateError(e))?;
        }

        Ok(())
    }
}
