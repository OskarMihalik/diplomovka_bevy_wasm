use axum::{
    body::Body,
    extract::{connect_info::ResponseFuture, Path, State},
    Json,
};
use axum_macros::debug_handler;
use cornucopia_async::Params;
use dto::{
    default::ErrorDto,
    users::{
        GetUsersDto, OtherUserDto, OtherUserDtoResponse, OtherUsersDtoResponse, ProjectUserDto,
        ProjectUsersDtoResponse, UserToProjectDto,
    },
};
use model::cornucopia::queries::{
    auth::select_user,
    tags::InsertProjectParams,
    users::{
        insert_project_user, select_user_by_id, select_users, select_users_in_project,
        InsertProjectUserParams, SelectUsersInProjectParams,
    },
};
use tokio_postgres::{Client, GenericClient};

use crate::{auth::claim::Claims, ConnectionPool};

use super::utils::{map_err_pool_con, map_sql_error};

#[debug_handler]
pub async fn get_users_service(
    _claims: Claims,
    State(pool): State<ConnectionPool>,
    Json(dto): Json<GetUsersDto>,
) -> Json<OtherUsersDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    return Json(get_users(&dto.email, connection.client()).await);
}

#[debug_handler]
pub async fn add_user_to_project_service(
    claims: Claims,
    State(pool): State<ConnectionPool>,
    Json(dto): Json<UserToProjectDto>,
) -> Json<OtherUserDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = insert_project_user()
        .params(
            connection.client(),
            &InsertProjectUserParams {
                project_id: dto.project_id,
                user_id: dto.user_id,
                is_admin: false,
            },
        )
        .await;

    match result {
        Ok(count) => {
            if count == 0 {
                return Json(Err(ErrorDto::new("User not inserted".to_string())));
            }
        }
        Err(err) => return map_sql_error(err),
    }

    return Json(get_user(dto.user_id, connection.client()).await);
}

pub async fn get_users_in_project_service(
    claims: Claims,
    State(pool): State<ConnectionPool>,
    Path(project_id): Path<i32>,
) -> Json<ProjectUsersDtoResponse> {
    let connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return map_err_pool_con(error),
    };

    let result = select_users_in_project()
        .params(
            connection.client(),
            &SelectUsersInProjectParams {
                project_id: project_id,
                limit: 100,
                offset: 0,
            },
        )
        .all()
        .await;

    match result {
        Ok(ok) => {
            return Json(Ok(ok
                .iter()
                .map(|user| ProjectUserDto {
                    id: user.id,
                    email: user.email.clone(),
                    username: user.username.clone(),
                    is_admin: user.is_admin,
                })
                .collect()))
        }
        Err(err) => return map_sql_error(err),
    }
}

pub async fn get_users(email: &str, client: &Client) -> Result<Vec<OtherUserDto>, ErrorDto> {
    let result = select_users().bind(client, &email, &100, &0).all().await;
    match result {
        Ok(rows) => {
            let dtos: Vec<OtherUserDto> = rows
                .into_iter()
                .map(|row| OtherUserDto {
                    id: row.id,
                    email: row.email,
                    username: row.username,
                })
                .collect();
            return Ok(dtos);
        }
        Err(error) => return Err(ErrorDto::new(format!("{:?}", error))),
    }
}

pub async fn get_user(id: i32, client: &Client) -> Result<OtherUserDto, ErrorDto> {
    let result = select_user_by_id().bind(client, &id).one().await;
    match result {
        Ok(row) => {
            let dto = OtherUserDto {
                id: row.id,
                email: row.email,
                username: row.username,
            };
            return Ok(dto);
        }
        Err(error) => return Err(ErrorDto::new(format!("{:?}", error))),
    }
}
