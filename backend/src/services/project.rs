use crate::{auth::claim::Claims, ConnectionPool};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use cornucopia_async::Params;
use dto::project::{NewProjectDto, ProjectDto, ProjectDtoResponse, ProjectsDtoResponse};
use model::cornucopia::queries::{
    tags::{insert_project, select_project, select_projects},
    users::{insert_project_user, InsertProjectUserParams},
};
use tokio_postgres::{Client, GenericClient};

use super::utils::{map_err_pool_con, map_error_reason, map_sql_error};

#[debug_handler]
pub async fn get_project_service(
    claims: Claims,
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
pub async fn get_projects_service(
    claims: Claims,
    State(pool): State<ConnectionPool>,
) -> Json<ProjectsDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    return get_projects(connection.client()).await;
}

pub async fn get_projects(client: &Client) -> Json<ProjectsDtoResponse> {
    let result = match select_projects().bind(client, &100, &0).all().await {
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

#[debug_handler]
pub async fn insert_project_service(
    claims: Claims,
    State(pool): State<ConnectionPool>,
    Json(dto): Json<NewProjectDto>,
) -> Json<ProjectsDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let new_project_id = match insert_project()
        .bind(connection.client(), &dto.name, &dto.description, &claims.id)
        .one()
        .await
    {
        Ok(ok) => ok,
        Err(error) => return map_sql_error(error),
    };

    let result = insert_project_user()
        .params(
            connection.client(),
            &InsertProjectUserParams {
                project_id: new_project_id,
                user_id: claims.id,
                is_admin: true,
            },
        )
        .await;

    match result {
        Ok(_) => (),
        Err(error) => return map_sql_error(error),
    }

    return get_projects(connection.client()).await;
}
