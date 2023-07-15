use anyhow::Context;

use super::model::{Project, ProjectCreate};

pub async fn create_project(
    pool: &sqlx::PgPool,
    project_create: ProjectCreate,
) -> Result<Project, anyhow::Error> {
    let sql = "INSERT INTO projects (name) VALUES ($1) RETURNING *";

    let created_project = sqlx::query_as::<_, Project>(sql)
        .bind(project_create.name)
        .fetch_one(pool)
        .await
        .context("Failed to insert a new project.")?;

    Ok(created_project)
}
