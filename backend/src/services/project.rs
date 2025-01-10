use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use dto::{
    default::ErrorDto,
    project::{ProjectDto, ProjectDtoResponse, ProjectsDtoResponse},
};
use model::cornucopia::queries::tags::{select_project, select_projects};
use tokio_postgres::GenericClient;

use crate::ConnectionPool;

use super::utils::{map_err_pool_con, map_sql_error};

#[debug_handler]
pub async fn get_project_service(
    Path(project_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<ProjectDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = match select_project()
        .bind(connection.client(), &project_id)
        .one()
        .await
    {
        Ok(ok) => ok,
        Err(error) => return map_sql_error(error),
    };

    let dto = ProjectDto {
        id: result.id,
        name: result.name.clone(),
        description: result.description.clone(),
        created_at: result.created_at,
        updated_at: result.updated_at,
    };

    return Json(ProjectDtoResponse::Ok(dto));
}

#[debug_handler]
pub async fn get_projects_service(State(pool): State<ConnectionPool>) -> Json<ProjectsDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = match select_projects()
        .bind(connection.client(), &100, &0)
        .all()
        .await
    {
        Ok(ok) => ok,
        Err(error) => return map_sql_error(error),
    };

    let dto: Vec<ProjectDto> = result
        .iter()
        .map(|project| ProjectDto {
            id: project.id,
            name: project.name.clone(),
            description: project.description.clone(),
            created_at: project.created_at,
            updated_at: project.updated_at,
        })
        .collect();

    return Json(ProjectsDtoResponse::Ok(dto));
}
