use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use dto::{
    default::ErrorDto,
    project::{ProjectDto, ProjectDtoResponse},
};
use model::cornucopia::queries::tags::select_project;
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
