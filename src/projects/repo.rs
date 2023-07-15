use anyhow::Context;

use super::model::{Project, ProjectCreate};

pub async fn get_all_projects(pool: &sqlx::PgPool) -> Result<Vec<Project>, anyhow::Error> {
    let sql = "SELECT * FROM projects";

    Ok(sqlx::query_as::<_, Project>(sql).fetch_all(pool).await?)
}

pub async fn create_project(
    pool: &sqlx::PgPool,
    project_create: ProjectCreate,
) -> Result<Project, anyhow::Error> {
    let sql = "INSERT INTO projects (name) VALUES ($1) RETURNING *";

    Ok(sqlx::query_as::<_, Project>(sql)
        .bind(project_create.name)
        .fetch_one(pool)
        .await
        .context("Failed to insert a new project.")?)
}
